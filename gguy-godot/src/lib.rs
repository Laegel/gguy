use std::time::Instant;

use gguy_core::input::{PointerButton, PointerCoords};
use gguy_core::log::{LogLevel, Logger};
use gguy_core::{Document, InputEvent, RenderThread};
use gguy_core::traits::EventSink;
#[cfg(feature = "vulkan-interop")]
use gguy_core::GpuTextureOutput;
use godot::builtin::{GString, VarDictionary};
#[cfg(not(feature = "vulkan-interop"))]
use godot::classes::image::Format;
#[cfg(not(feature = "vulkan-interop"))]
use godot::classes::Image;
use godot::classes::{
    ImageTexture, Input as GodotInput, InputEvent as GodotInputEvent,
    InputEventKey, InputEventMouseButton, InputEventMouseMotion, INode,
    Texture2D,
};
#[cfg(feature = "vulkan-interop")]
use godot::classes::{RenderingServer, Texture2Drd};
use godot::global::MouseButton;
use godot::prelude::*;
#[cfg(feature = "vulkan-interop")]
use godot::builtin::Rid;
use keyboard_types::{Code, Modifiers};

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

fn build_modifiers(key: &InputEventKey) -> Modifiers {
    let mut mods = Modifiers::empty();
    if key.is_shift_pressed() {
        mods |= Modifiers::SHIFT;
    }
    if key.is_ctrl_pressed() {
        mods |= Modifiers::CONTROL;
    }
    if key.is_alt_pressed() {
        mods |= Modifiers::ALT;
    }
    if key.is_meta_pressed() {
        mods |= Modifiers::META;
    }
    mods
}

fn godot_key_to_code(key: &godot::global::Key) -> Code {
    use godot::global::Key;
    let ord = key.ord();
    // Non-printable keys (ord >= 0x400000)
    if ord >= 0x400000 {
        match ord {
            k if k == Key::ESCAPE.ord() => Code::Escape,
            k if k == Key::TAB.ord() => Code::Tab,
            k if k == Key::BACKTAB.ord() => Code::Tab,
            k if k == Key::BACKSPACE.ord() => Code::Backspace,
            k if k == Key::ENTER.ord() => Code::Enter,
            k if k == Key::KP_ENTER.ord() => Code::NumpadEnter,
            k if k == Key::INSERT.ord() => Code::Insert,
            k if k == Key::DELETE.ord() => Code::Delete,
            k if k == Key::PAUSE.ord() => Code::Pause,
            k if k == Key::PRINT.ord() => Code::PrintScreen,
            k if k == Key::HOME.ord() => Code::Home,
            k if k == Key::END.ord() => Code::End,
            k if k == Key::LEFT.ord() => Code::ArrowLeft,
            k if k == Key::UP.ord() => Code::ArrowUp,
            k if k == Key::RIGHT.ord() => Code::ArrowRight,
            k if k == Key::DOWN.ord() => Code::ArrowDown,
            k if k == Key::PAGEUP.ord() => Code::PageUp,
            k if k == Key::PAGEDOWN.ord() => Code::PageDown,
            k if k == Key::SHIFT.ord() => Code::ShiftLeft,
            k if k == Key::CTRL.ord() => Code::ControlLeft,
            k if k == Key::META.ord() => Code::MetaLeft,
            k if k == Key::ALT.ord() => Code::AltLeft,
            k if k == Key::CAPSLOCK.ord() => Code::CapsLock,
            k if k == Key::NUMLOCK.ord() => Code::NumLock,
            k if k == Key::SCROLLLOCK.ord() => Code::ScrollLock,
            k if k == Key::F1.ord() => Code::F1,
            k if k == Key::F2.ord() => Code::F2,
            k if k == Key::F3.ord() => Code::F3,
            k if k == Key::F4.ord() => Code::F4,
            k if k == Key::F5.ord() => Code::F5,
            k if k == Key::F6.ord() => Code::F6,
            k if k == Key::F7.ord() => Code::F7,
            k if k == Key::F8.ord() => Code::F8,
            k if k == Key::F9.ord() => Code::F9,
            k if k == Key::F10.ord() => Code::F10,
            k if k == Key::F11.ord() => Code::F11,
            k if k == Key::F12.ord() => Code::F12,
            k if k == Key::F13.ord() => Code::F13,
            k if k == Key::F14.ord() => Code::F14,
            k if k == Key::F15.ord() => Code::F15,
            k if k == Key::F16.ord() => Code::F16,
            k if k == Key::F17.ord() => Code::F17,
            k if k == Key::F18.ord() => Code::F18,
            k if k == Key::F19.ord() => Code::F19,
            k if k == Key::F20.ord() => Code::F20,
            k if k == Key::F21.ord() => Code::F21,
            k if k == Key::F22.ord() => Code::F22,
            k if k == Key::F23.ord() => Code::F23,
            k if k == Key::F24.ord() => Code::F24,
            k if k == Key::F25.ord() => Code::F24,
            k if k == Key::F26.ord() => Code::F24,
            k if k == Key::F27.ord() => Code::F24,
            k if k == Key::F28.ord() => Code::F24,
            k if k == Key::F29.ord() => Code::F24,
            k if k == Key::F30.ord() => Code::F24,
            k if k == Key::F31.ord() => Code::F24,
            k if k == Key::F32.ord() => Code::F24,
            k if k == Key::F33.ord() => Code::F24,
            k if k == Key::F34.ord() => Code::F24,
            k if k == Key::F35.ord() => Code::F24,
            k if k == Key::KP_MULTIPLY.ord() => Code::NumpadMultiply,
            k if k == Key::KP_DIVIDE.ord() => Code::NumpadDivide,
            k if k == Key::KP_SUBTRACT.ord() => Code::NumpadSubtract,
            k if k == Key::KP_PERIOD.ord() => Code::NumpadDecimal,
            k if k == Key::KP_ADD.ord() => Code::NumpadAdd,
            k if k == Key::KP_0.ord() => Code::Numpad0,
            k if k == Key::KP_1.ord() => Code::Numpad1,
            k if k == Key::KP_2.ord() => Code::Numpad2,
            k if k == Key::KP_3.ord() => Code::Numpad3,
            k if k == Key::KP_4.ord() => Code::Numpad4,
            k if k == Key::KP_5.ord() => Code::Numpad5,
            k if k == Key::KP_6.ord() => Code::Numpad6,
            k if k == Key::KP_7.ord() => Code::Numpad7,
            k if k == Key::KP_8.ord() => Code::Numpad8,
            k if k == Key::KP_9.ord() => Code::Numpad9,
            k if k == Key::MENU.ord() => Code::ContextMenu,
            k if k == Key::HELP.ord() => Code::Help,
            k if k == Key::BACK.ord() => Code::BrowserBack,
            k if k == Key::FORWARD.ord() => Code::BrowserForward,
            k if k == Key::STOP.ord() => Code::BrowserStop,
            k if k == Key::REFRESH.ord() => Code::BrowserRefresh,
            k if k == Key::VOLUMEDOWN.ord() => Code::AudioVolumeDown,
            k if k == Key::VOLUMEMUTE.ord() => Code::AudioVolumeMute,
            k if k == Key::VOLUMEUP.ord() => Code::AudioVolumeUp,
            k if k == Key::MEDIAPLAY.ord() => Code::MediaPlay,
            k if k == Key::MEDIASTOP.ord() => Code::MediaStop,
            k if k == Key::MEDIAPREVIOUS.ord() => Code::MediaTrackPrevious,
            k if k == Key::MEDIANEXT.ord() => Code::MediaTrackNext,
            k if k == Key::SEARCH.ord() => Code::BrowserSearch,
            k if k == Key::HOMEPAGE.ord() => Code::BrowserHome,
            k if k == Key::LAUNCHMAIL.ord() => Code::LaunchMail,
            k if k == Key::STANDBY.ord() => Code::Unidentified,
            _ => Code::Unidentified,
        }
    } else {
        match ord {
            32 => Code::Space,
            48 => Code::Digit0,
            49 => Code::Digit1,
            50 => Code::Digit2,
            51 => Code::Digit3,
            52 => Code::Digit4,
            53 => Code::Digit5,
            54 => Code::Digit6,
            55 => Code::Digit7,
            56 => Code::Digit8,
            57 => Code::Digit9,
            65 => Code::KeyA,
            66 => Code::KeyB,
            67 => Code::KeyC,
            68 => Code::KeyD,
            69 => Code::KeyE,
            70 => Code::KeyF,
            71 => Code::KeyG,
            72 => Code::KeyH,
            73 => Code::KeyI,
            74 => Code::KeyJ,
            75 => Code::KeyK,
            76 => Code::KeyL,
            77 => Code::KeyM,
            78 => Code::KeyN,
            79 => Code::KeyO,
            80 => Code::KeyP,
            81 => Code::KeyQ,
            82 => Code::KeyR,
            83 => Code::KeyS,
            84 => Code::KeyT,
            85 => Code::KeyU,
            86 => Code::KeyV,
            87 => Code::KeyW,
            88 => Code::KeyX,
            89 => Code::KeyY,
            90 => Code::KeyZ,
            _ => Code::Unidentified,
        }
    }
}

struct GodotEventSink;

impl EventSink for GodotEventSink {
    fn navigate(&mut self, url: &str) {
        godot::classes::Os::singleton().shell_open(url);
    }

    fn set_cursor(&mut self, _cursor: &str) {
        // Cursor changes are handled via Input singleton in process().
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
    vp_width: u32,
    vp_height: u32,
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
        doc.set_event_sink(Box::new(GodotEventSink));
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

    #[func]
    pub fn create_element(&mut self, parent_selector: GString, tag: GString, attrs: VarDictionary) -> i64 {
        api::layer1_mutation::create_element(self, &parent_selector.to_string(), &tag.to_string(), &attrs)
    }

    #[func]
    pub fn remove_element(&mut self, selector: GString) {
        api::layer1_mutation::remove_element(self, &selector.to_string());
    }

    // ── InputMap action queries ───────────────────────────────────────────

    #[func]
    pub fn is_action_pressed(&self, action: GString) -> bool {
        GodotInput::singleton().is_action_pressed(action.to_string().as_str())
    }

    #[func]
    pub fn is_action_just_pressed(&self, action: GString) -> bool {
        GodotInput::singleton().is_action_just_pressed(action.to_string().as_str())
    }

    #[func]
    pub fn is_action_just_released(&self, action: GString) -> bool {
        GodotInput::singleton().is_action_just_released(action.to_string().as_str())
    }

    #[func]
    pub fn get_action_strength(&self, action: GString) -> f64 {
        GodotInput::singleton().get_action_strength(action.to_string().as_str()) as f64
    }

    #[func]
    pub fn get_axis(&self, negative_action: GString, positive_action: GString) -> f64 {
        GodotInput::singleton().get_axis(
            negative_action.to_string().as_str(),
            positive_action.to_string().as_str(),
        ) as f64
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

        godot_print!("[debug] needs_repaint={}", needs_repaint);
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

impl GguySurface {
    fn refresh_viewport_size(&mut self) {
        if let Some(vp) = self.base().get_viewport() {
            let rect = vp.get_visible_rect();
            self.vp_width = rect.size.x.max(1.0) as u32;
            self.vp_height = rect.size.y.max(1.0) as u32;
        }
        godot_print!(
            "[input] viewport=({}, {})  doc=({}, {})  scale={}",
            self.vp_width, self.vp_height,
            self.width, self.height,
            self.scale,
        );
    }

    fn scale_coords(&self, vp_x: f64, vp_y: f64) -> PointerCoords {
        let phys_w = self.width as f64 * self.scale;
        let phys_h = self.height as f64 * self.scale;
        // Texture is centered in the TextureRect (STRETCH_KEEP).
        // Compute the texture's top-left corner in viewport space.
        let offset_x = (self.vp_width as f64 - phys_w) / 2.0;
        let offset_y = (self.vp_height as f64 - phys_h) / 2.0;
        // Map viewport coords → texture pixel coords → document logical coords.
        PointerCoords {
            x: (vp_x - offset_x) / self.scale,
            y: (vp_y - offset_y) / self.scale,
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
            vp_width: 800,
            vp_height: 600,
        }
    }

    fn input(&mut self, event: Gd<GodotInputEvent>) {
        godot_print!("[input] event={}", event.get_class());

        if self.document.is_none() {
            return;
        }

        if let Ok(mb) = event.clone().try_cast::<InputEventMouseButton>() {
            let btn = mb.get_button_index();
            let pos = mb.get_position();
            let coords = self.scale_coords(pos.x as f64, pos.y as f64);
            godot_print!("[input] MouseButton raw=({}, {}) scaled=({}, {})",
                pos.x, pos.y, coords.x, coords.y);
            let doc = self.document.as_mut().unwrap();
            let modifiers = {
                let mut mods = Modifiers::empty();
                if mb.is_shift_pressed() { mods |= Modifiers::SHIFT; }
                if mb.is_ctrl_pressed() { mods |= Modifiers::CONTROL; }
                if mb.is_alt_pressed() { mods |= Modifiers::ALT; }
                if mb.is_meta_pressed() { mods |= Modifiers::META; }
                mods
            };

            let ord = btn.ord();
            if ord == MouseButton::WHEEL_UP.ord() {
                doc.handle_input_event(&InputEvent::Scroll {
                    delta_x: 0.0,
                    delta_y: 1.0,
                    modifiers,
                });
            } else if ord == MouseButton::WHEEL_DOWN.ord() {
                doc.handle_input_event(&InputEvent::Scroll {
                    delta_x: 0.0,
                    delta_y: -1.0,
                    modifiers,
                });
            } else if ord == MouseButton::WHEEL_LEFT.ord() {
                doc.handle_input_event(&InputEvent::Scroll {
                    delta_x: -1.0,
                    delta_y: 0.0,
                    modifiers,
                });
            } else if ord == MouseButton::WHEEL_RIGHT.ord() {
                doc.handle_input_event(&InputEvent::Scroll {
                    delta_x: 1.0,
                    delta_y: 0.0,
                    modifiers,
                });
            } else if mb.is_pressed() {
                doc.handle_input_event(&InputEvent::PointerDown {
                    coords,
                    button: map_mouse_button(btn),
                    modifiers,
                });
            } else {
                doc.handle_input_event(&InputEvent::PointerUp {
                    coords,
                    button: map_mouse_button(btn),
                    modifiers,
                });
            }
            return;
        }

        if let Ok(mm) = event.clone().try_cast::<InputEventMouseMotion>() {
            let pos = mm.get_position();
            let modifiers = {
                let mut mods = Modifiers::empty();
                if mm.is_shift_pressed() { mods |= Modifiers::SHIFT; }
                if mm.is_ctrl_pressed() { mods |= Modifiers::CONTROL; }
                if mm.is_alt_pressed() { mods |= Modifiers::ALT; }
                if mm.is_meta_pressed() { mods |= Modifiers::META; }
                mods
            };
            let coords = self.scale_coords(pos.x as f64, pos.y as f64);
            godot_print!("[input] MouseMotion raw=({}, {}) scaled=({}, {})",
                pos.x, pos.y, coords.x, coords.y);
            let doc = self.document.as_mut().unwrap();
            doc.handle_input_event(&InputEvent::PointerMove {
                coords,
                modifiers,
            });
            return;
        }

        if let Ok(key) = event.clone().try_cast::<InputEventKey>() {
            if key.is_echo() {
                return;
            }
            let modifiers = build_modifiers(&key);
            let code = godot_key_to_code(&key.get_physical_keycode());
            let unicode = key.get_unicode();
            let key_str = if unicode > 0 {
                char::from_u32(unicode)
                    .map(|c| c.to_string())
                    .unwrap_or_default()
            } else {
                key.as_text_keycode().to_string()
            };

            let doc = self.document.as_mut().unwrap();
            if key.is_pressed() {
                doc.handle_input_event(&InputEvent::KeyDown {
                    key: key_str.clone(),
                    code,
                    modifiers,
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
                doc.handle_input_event(&InputEvent::KeyUp {
                    key: key_str,
                    code,
                    modifiers,
                });
            }
        }
    }

    fn process(&mut self, _delta: f64) {
        self.refresh_viewport_size();
        self.update_texture();
    }
}
