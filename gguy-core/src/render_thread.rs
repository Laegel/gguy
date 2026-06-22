use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use vello::Scene;
use wgpu::TextureUsages;
use wgpu_context::{BufferRenderer, BufferRendererConfig, WGPUContext};

use crate::output::{GpuTextureOutput, RenderOutput};

// Number of ring-buffer texture slots for GPU interop.
#[cfg(feature = "vulkan-interop")]
const RING_SLOTS: usize = 3;

struct ThreadedRenderer {
    buffer_renderer: BufferRenderer,
    vello_renderer: vello::Renderer,
    #[cfg(feature = "vulkan-interop")]
    ring_textures: Vec<(wgpu::Texture, wgpu::TextureView)>,
    #[cfg(feature = "vulkan-interop")]
    current_slot: usize,
    width: u32,
    height: u32,
}

impl ThreadedRenderer {
    fn new(width: u32, height: u32) -> Self {
        let mut context = WGPUContext::new();
        let buffer_renderer = pollster::block_on(
            context.create_buffer_renderer(BufferRendererConfig {
                width,
                height,
                usage: TextureUsages::STORAGE_BINDING,
            }),
        )
        .expect("Failed to create buffer renderer");

        let vello_renderer = vello::Renderer::new(
            buffer_renderer.device(),
            vello::RendererOptions {
                use_cpu: false,
                num_init_threads: None,
                antialiasing_support: vello::AaSupport::area_only(),
                pipeline_cache: None,
            },
        )
        .expect("Failed to create vello renderer");

        #[cfg(feature = "vulkan-interop")]
        let ring_textures = Self::create_ring(&buffer_renderer, width, height);

        Self {
            buffer_renderer,
            vello_renderer,
            #[cfg(feature = "vulkan-interop")]
            ring_textures,
            #[cfg(feature = "vulkan-interop")]
            current_slot: 0,
            width,
            height,
        }
    }

    #[cfg(feature = "vulkan-interop")]
    fn create_ring(
        buffer_renderer: &BufferRenderer,
        width: u32,
        height: u32,
    ) -> Vec<(wgpu::Texture, wgpu::TextureView)> {
        let mut ring = Vec::with_capacity(RING_SLOTS);
        for _ in 0..RING_SLOTS {
            let texture = buffer_renderer
                .device()
                .create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::STORAGE_BINDING
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            ring.push((texture, view));
        }
        ring
    }

    fn render_scene(&mut self, scene: &Scene, buffer: &mut [u8]) {
        let size = self.buffer_renderer.size();
        self.vello_renderer
            .render_to_texture(
                self.buffer_renderer.device(),
                self.buffer_renderer.queue(),
                scene,
                &self.buffer_renderer.target_texture_view(),
                &vello::RenderParams {
                    base_color: vello::peniko::Color::TRANSPARENT,
                    width: size.width,
                    height: size.height,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .expect("Vello render failed");

        self.buffer_renderer.copy_texture_to_buffer(buffer);
    }

    #[cfg(feature = "vulkan-interop")]
    fn render_scene_gpu(&mut self, scene: &Scene) -> GpuTextureOutput {
        let texture = &self.ring_textures[self.current_slot];
        let size = self.buffer_renderer.size();

        self.vello_renderer
            .render_to_texture(
                self.buffer_renderer.device(),
                self.buffer_renderer.queue(),
                scene,
                &texture.1,
                &vello::RenderParams {
                    base_color: vello::peniko::Color::TRANSPARENT,
                    width: size.width,
                    height: size.height,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .expect("Vello render failed");

        // Ensure all GPU work is submitted and complete before extracting the handle.
        self.buffer_renderer.queue().submit([]);
        self.buffer_renderer
            .device()
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("wgpu poll failed");

        // Extract raw VkImage via wgpu-hal interop.
        let vk_image: u64 = unsafe {
            use wgpu::hal as wgpu_hal;
            let guard = texture
                .0
                .as_hal::<wgpu_hal::api::Vulkan>()
                .expect("Vulkan backend required for GPU interop");
            let hal_tex: &wgpu_hal::vulkan::Texture = &guard;
            let raw = hal_tex.raw_handle();
            // vk::Image is #[repr(transparent)] wrapping u64.
            std::mem::transmute::<_, u64>(raw)
        };

        // Advance to next ring slot.
        self.current_slot = (self.current_slot + 1) % RING_SLOTS;

        GpuTextureOutput::new(vk_image, self.width, self.height)
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.buffer_renderer.resize(width, height);
        #[cfg(feature = "vulkan-interop")]
        {
            self.ring_textures = Self::create_ring(&self.buffer_renderer, width, height);
            self.current_slot = 0;
        }
        self.width = width;
        self.height = height;
    }
}

enum RenderCommand {
    Render(Scene, u32, u32),
    #[cfg_attr(not(feature = "vulkan-interop"), allow(dead_code))]
    RenderGpu(Scene, u32, u32),
    Resize(u32, u32),
    Shutdown,
}

pub struct RenderThread {
    cmd_tx: mpsc::Sender<RenderCommand>,
    result_rx: mpsc::Receiver<RenderOutput>,
    gpu_result_rx: mpsc::Receiver<GpuTextureOutput>,
    handle: Option<JoinHandle<()>>,
}

impl RenderThread {
    pub fn new(width: u32, height: u32) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel::<RenderCommand>();
        let (result_tx, result_rx) = mpsc::channel::<RenderOutput>();
        #[cfg_attr(not(feature = "vulkan-interop"), allow(unused))]
        let (gpu_result_tx, gpu_result_rx) = mpsc::channel::<GpuTextureOutput>();

        let handle = thread::spawn(move || {
            let mut renderer = ThreadedRenderer::new(width, height);

            while let Ok(cmd) = cmd_rx.recv() {
                match cmd {
                    RenderCommand::Render(scene, w, h) => {
                        let size = w as usize * h as usize * 4;
                        let mut buffer = vec![0u8; size];

                        renderer.render_scene(&scene, &mut buffer);

                        let _ = result_tx.send(RenderOutput::new(buffer, w, h));
                    }
                    #[cfg(feature = "vulkan-interop")]
                    RenderCommand::RenderGpu(scene, _w, _h) => {
                        let output = renderer.render_scene_gpu(&scene);
                        let _ = gpu_result_tx.send(output);
                    }
                    #[cfg(not(feature = "vulkan-interop"))]
                    RenderCommand::RenderGpu(scene, w, h) => {
                        let size = w as usize * h as usize * 4;
                        let mut buffer = vec![0u8; size];
                        renderer.render_scene(&scene, &mut buffer);
                        let _ = result_tx.send(RenderOutput::new(buffer, w, h));
                    }
                    RenderCommand::Resize(w, h) => {
                        renderer.resize(w, h);
                    }
                    RenderCommand::Shutdown => break,
                }
            }
        });

        Self {
            cmd_tx,
            result_rx,
            gpu_result_rx,
            handle: Some(handle),
        }
    }

    pub fn send_scene(&self, scene: Scene, width: u32, height: u32) {
        let _ = self.cmd_tx.send(RenderCommand::Render(scene, width, height));
    }

    #[cfg(feature = "vulkan-interop")]
    pub fn send_scene_gpu(&self, scene: Scene, width: u32, height: u32) {
        let _ = self
            .cmd_tx
            .send(RenderCommand::RenderGpu(scene, width, height));
    }

    pub fn try_recv(&mut self) -> Option<RenderOutput> {
        self.result_rx.try_recv().ok()
    }

    pub fn try_recv_gpu(&mut self) -> Option<GpuTextureOutput> {
        self.gpu_result_rx.try_recv().ok()
    }

    pub fn resize(&self, width: u32, height: u32) {
        let _ = self.cmd_tx.send(RenderCommand::Resize(width, height));
    }
}

impl Drop for RenderThread {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(RenderCommand::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
