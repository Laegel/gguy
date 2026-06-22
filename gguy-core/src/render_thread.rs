use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use vello::Scene;
use wgpu::TextureUsages;
use wgpu_context::{BufferRenderer, BufferRendererConfig, WGPUContext};

use crate::output::RenderOutput;

struct ThreadedRenderer {
    buffer_renderer: BufferRenderer,
    vello_renderer: vello::Renderer,
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

        Self {
            buffer_renderer,
            vello_renderer,
            width,
            height,
        }
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

    fn resize(&mut self, width: u32, height: u32) {
        self.buffer_renderer.resize(width, height);
        self.width = width;
        self.height = height;
    }
}

enum RenderCommand {
    Render(Scene, u32, u32),
    Resize(u32, u32),
    Shutdown,
}

pub struct RenderThread {
    cmd_tx: mpsc::Sender<RenderCommand>,
    result_rx: mpsc::Receiver<RenderOutput>,
    handle: Option<JoinHandle<()>>,
}

impl RenderThread {
    pub fn new(width: u32, height: u32) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel::<RenderCommand>();
        let (result_tx, result_rx) = mpsc::channel::<RenderOutput>();

        let handle = thread::spawn(move || {
            let mut renderer = ThreadedRenderer::new(width, height);

            while let Ok(cmd) = cmd_rx.recv() {
                match cmd {
                    RenderCommand::Render(scene, w, h) => {
                        let phys_w = w;
                        let phys_h = h;

                        let size = phys_w as usize * phys_h as usize * 4;
                        let mut buffer = vec![0u8; size];

                        renderer.render_scene(&scene, &mut buffer);

                        let _ = result_tx.send(RenderOutput::new(buffer, phys_w, phys_h));
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
            handle: Some(handle),
        }
    }

    pub fn send_scene(&self, scene: Scene, width: u32, height: u32) {
        let _ = self.cmd_tx.send(RenderCommand::Render(scene, width, height));
    }

    pub fn try_recv(&mut self) -> Option<RenderOutput> {
        self.result_rx.try_recv().ok()
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
