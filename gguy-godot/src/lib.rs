use gguy_core::input::{PointerButton, PointerCoords};
use gguy_core::{Document, InputEvent, RenderThread};
#[cfg(feature = "vulkan-interop")]
use gguy_core::GpuTextureOutput;
use godot::builtin::GString;
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
        let doc = Document::new(&html_str, self.width, self.height, self.scale);
        self.document = Some(doc);
        self.render_thread = Some(RenderThread::new(phys_w, phys_h));
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
        if let Some(ref rt) = self.render_thread {
            let phys_w = (self.width as f64 * self.scale) as u32;
            let phys_h = (self.height as f64 * self.scale) as u32;
            rt.resize(phys_w, phys_h);
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

    fn update_texture(&mut self) {
        #[cfg(feature = "vulkan-interop")]
        {
            // GPU hand-off path: poll for GPU texture output first.
            if let Some(gpu_output) = self
                .render_thread
                .as_mut()
                .and_then(|rt| rt.try_recv_gpu())
            {
                let tex = self.create_gpu_texture(gpu_output);
                self.gpu_texture = Some(tex);
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
                if output.is_empty() {
                    return;
                }

                let mut pba = PackedByteArray::new();
                pba.extend(output.bytes().iter().copied());

                let img = Image::create_from_data(
                    output.width() as i32,
                    output.height() as i32,
                    false,
                    Format::RGBA8,
                    &pba,
                );

                if let Some(image) = img {
                    let mut tex = ImageTexture::new_gd();
                    tex.set_image(&image);
                    self.texture = Some(tex);
                }
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
            let doc = self.document.as_mut().unwrap();
            doc.reify();
            let scene = doc.paint_scene();
            let phys_w = (self.width as f64 * self.scale) as u32;
            let phys_h = (self.height as f64 * self.scale) as u32;
            if let Some(rt) = self.render_thread.as_ref() {
                #[cfg(feature = "vulkan-interop")]
                rt.send_scene_gpu(scene, phys_w, phys_h);
                #[cfg(not(feature = "vulkan-interop"))]
                rt.send_scene(scene, phys_w, phys_h);
            }
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
