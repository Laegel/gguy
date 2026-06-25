use std::time::Instant;

use gguy_core::input::{PointerButton, PointerCoords};
use gguy_core::log::{LogLevel, Logger};
use gguy_core::{Document, InputEvent, RenderThread};
#[cfg(feature = "vulkan-interop")]
use gguy_core::GpuTextureOutput;
use godot::builtin::{GString, VarDictionary};
#[cfg(not(feature = "vulkan-interop"))]
use godot::classes::image::Format;
#[cfg(not(feature = "vulkan-interop"))]
use godot::classes::Image;
use godot::classes::{
    ImageTexture, InputEvent as GodotInputEvent,
    InputEventKey, InputEventMouseButton, InputEventMouseMotion, INode,
    Texture2D,
};
#[cfg(feature = "vulkan-interop")]
use godot::classes::{RenderingServer, Texture2Drd};
use godot::global::MouseButton;
use godot::prelude::*;
#[cfg(feature = "vulkan-interop")]
use godot::builtin::Rid;

mod api;

fn map_mouse_button(btn: MouseButton) -> PointerButton {
    let ord = btn.ord();
    if ord == MouseButton::LEFT.ord() {
        PointerButton::Primary
    } else if ord == MouseButton::RIGHT.ord() {
        PointerButton::Secondary
    } else if ord == MouseButton::MIDDLE.ord() {
        PointerButton::Middle
    } else if ord == MouseButton::XBUTTON1.ord() {
        PointerButton::Back
    } else if ord == MouseButton::XBUTTON2.ord() {
        PointerButton::Forward
    } else {
        PointerButton::Other(ord as u8)
    }
}

struct GodotLogger;

impl Logger for GodotLogger {
    fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Error => godot_error!("{}", message),
            LogLevel::Profile | LogLevel::Warn | LogLevel::Info => godot_print!("{}", message),
            LogLevel::Debug | LogLevel::Trace => {}
        }
    }
}

struct GguyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GguyExtension {}

#[derive(GodotClass)]
#[class(base = Node)]
pub struct GguySurface {
    base: Base<Node>,
    document: Option<Document>,
    render_thread: Option<RenderThread>,
    texture: Option<Gd<ImageTexture>>,
    #[cfg(feature = "vulkan-interop")]
    gpu_texture: Option<Gd<Texture2Drd>>,
    #[cfg(feature = "vulkan-interop")]
    old_rd_rid: Option<Rid>,
    width: u32,
    height: u32,
    scale: f64,
}

#[godot_api]
impl GguySurface {
    #[func]
    pub fn load_html(&mut self, html: GString) {
        let html_str = html.to_string();
        let phys_w = (self.width as f64 * self.scale) as u32;
        let phys_h = (self.height as f64 * self.scale) as u32;
        let mut doc = Document::new(&html_str, self.width, self.height, self.scale);
        doc.set_logger(Box::new(GodotLogger));
        self.document = Some(doc);
        self.render_thread = Some(RenderThread::new(phys_w, phys_h, Some(Box::new(GodotLogger))));
    }

    #[cfg(feature = "vulkan-interop")]
    fn create_gpu_texture(&mut self, output: GpuTextureOutput) -> Gd<Texture2Drd> {
        use godot::classes::rendering_device::{
            DataFormat, TextureSamples, TextureType, TextureUsageBits,
        };
        let mut rd = RenderingServer::singleton()
            .get_rendering_device()
            .expect("No RenderingDevice available");

        // Free previous texture RID to avoid leaking.
        if let Some(old_rd) = self.old_rd_rid.take() {
            rd.free_rid(old_rd);
        }

        let rd_rid = rd.texture_create_from_extension(
            TextureType::TYPE_2D,
            DataFormat::R8G8B8A8_UNORM,
            TextureSamples::SAMPLES_1,
            TextureUsageBits::STORAGE_BIT
                | TextureUsageBits::SAMPLING_BIT
                | TextureUsageBits::CAN_COPY_FROM_BIT,
            output.vk_image,
            output.width as u64,
            output.height as u64,
            1, // depth
            1, // layers
        );

        self.old_rd_rid = Some(rd_rid);

        let mut tex = Texture2Drd::new_gd();
        tex.set_texture_rd_rid(rd_rid);
        tex
    }

    #[func]
    pub fn set_size(&mut self, width: f64, height: f64) {
        self.width = width as u32;
        self.height = height as u32;
        if let Some(ref mut doc) = self.document {
            doc.resize(self.width, self.height, self.scale);
        }
        if let Some(ref mut rt) = self.render_thread {
            let phys_w = (self.width as f64 * self.scale) as u32;
            let phys_h = (self.height as f64 * self.scale) as u32;
            rt.resize_sync(phys_w, phys_h);
            // Drain already discarded stale results.  The old texture (at the
            // old size) lives for one more frame — a one-frame stutter — then
            // gets replaced naturally by the next render.
        }
    }

    #[func]
    pub fn get_texture(&self) -> Gd<Texture2D> {
        #[cfg(feature = "vulkan-interop")]
        if let Some(ref tex) = self.gpu_texture {
            return tex.clone().upcast();
        }
        self.texture
            .clone()
            .map_or_else(|| ImageTexture::new_gd().upcast(), |t| t.upcast())
    }

    #[cfg(feature = "vulkan-interop")]
    #[func]
    pub fn get_gpu_texture(&self) -> Gd<Texture2Drd> {
        self.gpu_texture.clone().unwrap_or_else(Texture2Drd::new_gd)
    }

    // ── Layer 0: VDOM diff layer ──────────────────────────────────────────

    #[func]
    pub fn render(&mut self, tree: VarDictionary) {
        api::layer0_vdom::render(self, tree);
    }

    #[func]
    pub fn render_into(&mut self, container_id: GString, tree: VarDictionary) {
        api::layer0_vdom::render_into(self, &container_id.to_string(), tree);
    }

    // ── Layer 1: Low-level mutation API ────────────────────────────────────

    #[func]
    pub fn set_text(&mut self, selector: GString, value: GString) {
        api::layer1_mutation::set_text(self, &selector.to_string(), &value.to_string());
    }

    #[func]
    pub fn set_attribute(&mut self, selector: GString, name: GString, value: GString) {
        api::layer1_mutation::set_attribute(self, &selector.to_string(), &name.to_string(), &value.to_string());
    }

    #[func]
    pub fn remove_attribute(&mut self, selector: GString, name: GString) {
        api::layer1_mutation::remove_attribute(self, &selector.to_string(), &name.to_string());
    }

    #[func]
    pub fn add_class(&mut self, selector: GString, class: GString) {
        api::layer1_mutation::add_class(self, &selector.to_string(), &class.to_string());
    }

    #[func]
    pub fn remove_class(&mut self, selector: GString, class: GString) {
        api::layer1_mutation::remove_class(self, &selector.to_string(), &class.to_string());
    }

    fn update_texture(&mut self) {
        if let Some(rt) = self.render_thread.as_mut() {
            rt.drain_logs();
        }
        let _t_total = Instant::now();
        let mut tex_created = false;

        #[cfg(feature = "vulkan-interop")]
        {
            // GPU hand-off path: poll for GPU texture output first.
            if let Some(gpu_output) = self
                .render_thread
                .as_mut()
                .and_then(|rt| rt.try_recv_gpu())
            {
                let t = Instant::now();
                let tex = self.create_gpu_texture(gpu_output);
                self.gpu_texture = Some(tex);
                godot_print!("[profile] texture_create: {}µs", t.elapsed().as_micros());
                tex_created = true;
            }
        }

        #[cfg(not(feature = "vulkan-interop"))]
        {
            // CPU readback path (fallback).
            if let Some(output) = self
                .render_thread
                .as_mut()
                .and_then(|rt| rt.try_recv())
            {
                let t = Instant::now();
                if output.is_empty() {
                    return;
                }

                let expected = output.width() as usize * output.height() as usize * 4;
                debug_assert_eq!(
                    output.bytes().len(),
                    expected,
                    "RenderOutput buffer size mismatch"
                );
                let expected_w = (self.width as f64 * self.scale) as u32;
                let expected_h = (self.height as f64 * self.scale) as u32;
                debug_assert_eq!(output.width(), expected_w, "output width mismatch");
                debug_assert_eq!(output.height(), expected_h, "output height mismatch");

                let pba = PackedByteArray::from(output.bytes());

                let img = Image::create_from_data(
                    output.width() as i32,
                    output.height() as i32,
                    false,
                    Format::RGBA8,
                    &pba,
                );

                if let Some(image) = img {
                    tex_created = true;
                    match self.texture.as_mut() {
                        Some(tex) => tex.update(&image),
                        None => {
                            let mut tex = ImageTexture::new_gd();
                            tex.set_image(&image);
                            self.texture = Some(tex);
                        }
                    }
                }

                godot_print!("[profile] texture_create: {}µs", t.elapsed().as_micros());
            }
        }

        // Check if we need to repaint / produce the initial frame.
        let needs_repaint = self
            .document
            .as_ref()
            .is_some_and(|doc| {
                #[cfg(feature = "vulkan-interop")]
                {
                    doc.needs_repaint() || self.gpu_texture.is_none()
                }
                #[cfg(not(feature = "vulkan-interop"))]
                {
                    doc.needs_repaint() || self.texture.is_none()
                }
            });

        if needs_repaint {
            let t = Instant::now();
            let doc = self.document.as_mut().unwrap();
            doc.reify();
            let reify_us = t.elapsed().as_micros();

            let t = Instant::now();
            let scene = doc.paint_scene();
            let paint_us = t.elapsed().as_micros();

            let phys_w = (self.width as f64 * self.scale) as u32;
            let phys_h = (self.height as f64 * self.scale) as u32;
            if let Some(rt) = self.render_thread.as_ref() {
                #[cfg(feature = "vulkan-interop")]
                rt.send_scene_gpu(scene, phys_w, phys_h);
                #[cfg(not(feature = "vulkan-interop"))]
                rt.send_scene(scene, phys_w, phys_h);
            }

            godot_print!("[profile] update_texture: total={}µs  reify={}µs  paint={}µs  tex={}",
                _t_total.elapsed().as_micros(), reify_us, paint_us, if tex_created { "yes" } else { "no" });
        }
    }
}

#[godot_api]
impl INode for GguySurface {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            document: None,
            render_thread: None,
            texture: None,
            #[cfg(feature = "vulkan-interop")]
            gpu_texture: None,
            #[cfg(feature = "vulkan-interop")]
            old_rd_rid: None,
            width: 800,
            height: 600,
            scale: 1.0,
        }
    }

    fn unhandled_input(&mut self, event: Gd<GodotInputEvent>) {
        let doc = match &mut self.document {
            Some(doc) => doc,
            None => return,
        };

        if let Ok(mb) = event.clone().try_cast::<InputEventMouseButton>() {
            let btn = mb.get_button_index();
            let pos = mb.get_position();
            let coords = PointerCoords {
                x: pos.x as f64,
                y: pos.y as f64,
            };

            let ord = btn.ord();
            if ord == MouseButton::WHEEL_UP.ord() {
                doc.handle_input_event(&InputEvent::Scroll {
                    delta_x: 0.0,
                    delta_y: 1.0,
                });
            } else if ord == MouseButton::WHEEL_DOWN.ord() {
                doc.handle_input_event(&InputEvent::Scroll {
                    delta_x: 0.0,
                    delta_y: -1.0,
                });
            } else if ord == MouseButton::WHEEL_LEFT.ord() {
                doc.handle_input_event(&InputEvent::Scroll {
                    delta_x: -1.0,
                    delta_y: 0.0,
                });
            } else if ord == MouseButton::WHEEL_RIGHT.ord() {
                doc.handle_input_event(&InputEvent::Scroll {
                    delta_x: 1.0,
                    delta_y: 0.0,
                });
            } else if mb.is_pressed() {
                doc.handle_input_event(&InputEvent::PointerDown {
                    coords,
                    button: map_mouse_button(btn),
                });
            } else {
                doc.handle_input_event(&InputEvent::PointerUp {
                    coords,
                    button: map_mouse_button(btn),
                });
            }
            return;
        }

        if let Ok(mm) = event.clone().try_cast::<InputEventMouseMotion>() {
            let pos = mm.get_position();
            doc.handle_input_event(&InputEvent::PointerMove {
                coords: PointerCoords {
                    x: pos.x as f64,
                    y: pos.y as f64,
                },
            });
            return;
        }

        if let Ok(key) = event.clone().try_cast::<InputEventKey>() {
            if key.is_echo() {
                return;
            }
            let unicode = key.get_unicode();
            let key_str = if unicode > 0 {
                char::from_u32(unicode)
                    .map(|c| c.to_string())
                    .unwrap_or_default()
            } else {
                key.as_text_keycode().to_string()
            };

            if key.is_pressed() {
                doc.handle_input_event(&InputEvent::KeyDown {
                    key: key_str.clone(),
                });
                if unicode > 0 {
                    if let Some(c) = char::from_u32(unicode) {
                        if !c.is_control() {
                            doc.handle_input_event(&InputEvent::TextInput {
                                text: c.to_string(),
                            });
                        }
                    }
                }
            } else {
                doc.handle_input_event(&InputEvent::KeyUp { key: key_str });
            }
        }
    }

    fn process(&mut self, _delta: f64) {
        self.update_texture();
    }
}
