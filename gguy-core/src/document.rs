use anyrender_vello::VelloScenePainter;
use blitz_dom::Document as _;
use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;
use blitz_traits::events::{
    BlitzKeyEvent, BlitzPointerEvent, BlitzWheelDelta, BlitzWheelEvent, MouseEventButton,
    PointerCoords, PointerDetails, UiEvent,
};
use blitz_traits::shell::{ColorScheme, Viewport};
use keyboard_types::{Code, Key, Location, Modifiers};
use vello::Scene;

use crate::input::{InputEvent, PointerButton};
use crate::traits::{EventSink, ResourceProvider};

pub struct Document {
    html_doc: HtmlDocument,
    width: u32,
    height: u32,
    scale: f64,
    resource_provider: Option<Box<dyn ResourceProvider>>,
    event_sink: Option<Box<dyn EventSink>>,
}

fn map_button(btn: PointerButton) -> MouseEventButton {
    match btn {
        PointerButton::Primary => MouseEventButton::Main,
        PointerButton::Secondary => MouseEventButton::Secondary,
        PointerButton::Middle => MouseEventButton::Auxiliary,
        PointerButton::Back => MouseEventButton::Fourth,
        PointerButton::Forward => MouseEventButton::Fifth,
        PointerButton::Other(_) => MouseEventButton::Main,
    }
}

fn make_coords(x: f64, y: f64) -> PointerCoords {
    PointerCoords {
        page_x: x as f32,
        page_y: y as f32,
        screen_x: x as f32,
        screen_y: y as f32,
        client_x: x as f32,
        client_y: y as f32,
    }
}

fn make_pointer_event(x: f64, y: f64, button: PointerButton) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: blitz_traits::events::BlitzPointerId::Mouse,
        is_primary: true,
        coords: make_coords(x, y),
        button: map_button(button),
        buttons: Default::default(),
        mods: Modifiers::empty(),
        details: PointerDetails {
            pressure: 0.5,
            tangential_pressure: 0.0,
            tilt_x: 0,
            tilt_y: 0,
            twist: 0,
            altitude: 0.0,
            azimuth: 0.0,
        },
    }
}

impl Document {
    pub fn new(html: &str, width: u32, height: u32, scale: f64) -> Self {
        let phys_w = (width as f64 * scale) as u32;
        let phys_h = (height as f64 * scale) as u32;

        let viewport = Viewport::new(phys_w, phys_h, scale as f32, ColorScheme::Light);

        let config = DocumentConfig {
            viewport: Some(viewport),
            base_url: Some("about:blank".to_string()),
            ..Default::default()
        };

        let mut html_doc = HtmlDocument::from_html(html, config);
        html_doc.resolve(0.0);

        Self {
            html_doc,
            width,
            height,
            scale,
            resource_provider: None,
            event_sink: None,
        }
    }

    pub fn set_resource_provider(&mut self, provider: Box<dyn ResourceProvider>) {
        self.resource_provider = Some(provider);
    }

    pub fn set_event_sink(&mut self, sink: Box<dyn EventSink>) {
        self.event_sink = Some(sink);
    }

    pub fn needs_repaint(&self) -> bool {
        self.html_doc.has_changes() || self.html_doc.is_animating()
    }

    pub fn reify(&mut self) {
        self.html_doc.resolve(0.0);
    }

    pub fn paint_scene(&mut self) -> Scene {
        let mut scene = Scene::new();
        let mut painter = VelloScenePainter::new(&mut scene);

        blitz_paint::paint_scene(
            &mut painter,
            &mut self.html_doc,
            self.scale,
            self.width,
            self.height,
            0,
            0,
        );

        scene
    }

    pub fn resize(&mut self, width: u32, height: u32, scale: f64) {
        self.width = width;
        self.height = height;
        self.scale = scale;

        let phys_w = (width as f64 * scale) as u32;
        let phys_h = (height as f64 * scale) as u32;

        let viewport = Viewport::new(phys_w, phys_h, scale as f32, ColorScheme::Light);
        self.html_doc.set_viewport(viewport);
        self.html_doc.resolve(0.0);
    }

    pub fn handle_input_event(&mut self, event: &InputEvent) {
        let ui_event = match *event {
            InputEvent::PointerMove { coords } => {
                let x = coords.x;
                let y = coords.y;
                self.html_doc.set_hover_to(x as f32, y as f32);
                UiEvent::PointerMove(BlitzPointerEvent {
                    id: blitz_traits::events::BlitzPointerId::Mouse,
                    is_primary: true,
                    coords: make_coords(x, y),
                    button: MouseEventButton::Main,
                    buttons: Default::default(),
                    mods: Modifiers::empty(),
                    details: PointerDetails {
                        pressure: 0.0,
                        tangential_pressure: 0.0,
                        tilt_x: 0,
                        tilt_y: 0,
                        twist: 0,
                        altitude: 0.0,
                        azimuth: 0.0,
                    },
                })
            }
            InputEvent::PointerDown { coords, button } => {
                UiEvent::PointerDown(make_pointer_event(coords.x, coords.y, button))
            }
            InputEvent::PointerUp { coords, button } => {
                UiEvent::PointerUp(make_pointer_event(coords.x, coords.y, button))
            }
            InputEvent::Scroll { delta_x, delta_y } => UiEvent::Wheel(BlitzWheelEvent {
                delta: BlitzWheelDelta::Pixels(delta_x as f64, delta_y as f64),
                coords: make_coords(0.0, 0.0),
                buttons: Default::default(),
                mods: Modifiers::empty(),
            }),
            InputEvent::KeyDown { ref key } => {
                let k = Key::Character(key.clone());
                UiEvent::KeyDown(BlitzKeyEvent {
                    key: k,
                    code: Code::Unidentified,
                    modifiers: Modifiers::empty(),
                    location: Location::Standard,
                    is_auto_repeating: false,
                    is_composing: false,
                    state: blitz_traits::events::KeyState::Pressed,
                    text: None,
                })
            }
            InputEvent::KeyUp { ref key } => {
                let k = Key::Character(key.clone());
                UiEvent::KeyUp(BlitzKeyEvent {
                    key: k,
                    code: Code::Unidentified,
                    modifiers: Modifiers::empty(),
                    location: Location::Standard,
                    is_auto_repeating: false,
                    is_composing: false,
                    state: blitz_traits::events::KeyState::Released,
                    text: None,
                })
            }
            InputEvent::TextInput { .. } => return,
        };

        self.html_doc.handle_ui_event(ui_event);
        self.html_doc.resolve(0.0);
    }
}
