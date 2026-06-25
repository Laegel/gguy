use std::fmt;
use std::time::Instant;

use anyrender_vello::VelloScenePainter;
use blitz_dom::Document as _;
use blitz_dom::DocumentConfig;
use blitz_html::HtmlDocument;

use crate::log::{LogLevel, Logger};
use blitz_traits::events::{
    BlitzKeyEvent, BlitzPointerEvent, BlitzWheelDelta, BlitzWheelEvent, MouseEventButton,
    PointerCoords, PointerDetails, UiEvent,
};
use blitz_traits::shell::{ColorScheme, Viewport};
use keyboard_types::{Code, Key, Location, Modifiers};
use blitz_dom::{LocalName, QualName, local_name, ns};
use vello::Scene;

use crate::input::{InputEvent, PointerButton};
use crate::traits::{EventSink, ResourceProvider};

pub struct Document {
    html_doc: HtmlDocument,
    width: u32,
    height: u32,
    scale: f64,
    needs_paint: bool,
    resource_provider: Option<Box<dyn ResourceProvider>>,
    event_sink: Option<Box<dyn EventSink>>,
    logger: Option<Box<dyn Logger>>,
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

fn qual(name: &str) -> QualName {
    QualName::new(None, ns!(html), LocalName::from(name))
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
            needs_paint: true,
            resource_provider: None,
            event_sink: None,
            logger: None,
        }
    }

    pub fn set_resource_provider(&mut self, provider: Box<dyn ResourceProvider>) {
        self.resource_provider = Some(provider);
    }

    pub fn set_event_sink(&mut self, sink: Box<dyn EventSink>) {
        self.event_sink = Some(sink);
    }

    pub fn set_logger(&mut self, logger: Box<dyn Logger>) {
        self.logger = Some(logger);
    }

    fn log_msg(&self, level: LogLevel, msg: fmt::Arguments<'_>) {
        if let Some(ref l) = self.logger {
            l.log(level, &fmt::format(msg));
        }
    }

    pub fn needs_repaint(&self) -> bool {
        let result = self.needs_paint || self.html_doc.has_changes() || self.html_doc.is_animating();
        self.log_msg(LogLevel::Trace, format_args!("[gguy] needs_repaint: needs_paint={} has_changes={} is_animating={} => {}", self.needs_paint, self.html_doc.has_changes(), self.html_doc.is_animating(), result));
        result
    }

    pub fn reify(&mut self) {
        let _t = Instant::now();

        self.needs_paint = false;
        self.html_doc.handle_messages();

        let t = Instant::now();
        self.html_doc.resolve_scroll_animation();
        let scroll_us = t.elapsed().as_micros();

        let t = Instant::now();
        self.html_doc.resolve_stylist(0.0);
        let style_us = t.elapsed().as_micros();

        let t = Instant::now();
        self.html_doc.resolve_layout_children();
        let construct_us = t.elapsed().as_micros();

        let t = Instant::now();
        self.html_doc.resolve_deferred_tasks();
        let deferred_us = t.elapsed().as_micros();

        let root_id = self.html_doc.root_element().id;
        let t = Instant::now();
        self.html_doc.flush_styles_to_layout(root_id);
        let flush_us = t.elapsed().as_micros();

        let t = Instant::now();
        self.html_doc.resolve_layout();
        let layout_us = t.elapsed().as_micros();

        self.log_msg(LogLevel::Profile, format_args!("[profile] reify: total={}µs  (scroll={} style={} construct={} deferred={} flush={} layout={})",
            _t.elapsed().as_micros(),
            scroll_us, style_us, construct_us, deferred_us, flush_us, layout_us));
    }

    pub fn paint_scene(&mut self) -> Scene {
        let _t = Instant::now();
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

        let elapsed = _t.elapsed();
        if elapsed.as_micros() > 200 {
            self.log_msg(LogLevel::Profile, format_args!("[profile] paint_scene: {}µs  ({}x{})", elapsed.as_micros(), self.width, self.height));
        }

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
        self.needs_paint = true;
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
        self.needs_paint = true;
    }

    // ── Query methods ──────────────────────────────────────────────────────

    pub fn get_element_by_id(&self, id: &str) -> Option<usize> {
        self.html_doc.get_element_by_id(id)
    }

    pub fn query_selector(&self, selector: &str) -> Option<usize> {
        self.html_doc.query_selector(selector).ok()?
    }

    // ── Layer 1 mutation primitives ────────────────────────────────────────
    //
    // `_by_node_id` variants skip the id-lookup step (caller already resolved).
    // `_by_id` variants are convenience wrappers for GDScript's Layer 1 API.

    pub fn set_text_by_id(&mut self, id: &str, text: &str) {
        let nid = self.html_doc.get_element_by_id(id);
        if let Some(node_id) = nid {
            self.set_text_by_node_id(node_id, text);
        }
    }

    pub fn set_text_by_node_id(&mut self, node_id: usize, text: &str) {
        let child_ids: Vec<usize> = self.html_doc
            .get_node(node_id)
            .map(|n| n.children.clone())
            .unwrap_or_default();

        let mut mutator = self.html_doc.mutate();
        if let Some(&text_id) = child_ids.first() {
            mutator.set_node_text(text_id, text);
        } else {
            let text_id = mutator.create_text_node(text);
            mutator.append_children(node_id, &[text_id]);
        }
        drop(mutator);
        self.needs_paint = true;
    }

    pub fn set_attribute_by_id(&mut self, id: &str, name: &str, value: &str) {
        let nid = self.html_doc.get_element_by_id(id);
        if let Some(node_id) = nid {
            self.set_attribute_by_node_id(node_id, name, value);
        }
    }

    pub fn set_attribute_by_node_id(&mut self, node_id: usize, name: &str, value: &str) {
        let mut mutator = self.html_doc.mutate();
        mutator.set_attribute(node_id, qual(name), value);
        drop(mutator);
        self.needs_paint = true;
    }

    pub fn remove_attribute_by_id(&mut self, id: &str, name: &str) {
        let nid = self.html_doc.get_element_by_id(id);
        if let Some(node_id) = nid {
            self.remove_attribute_by_node_id(node_id, name);
        }
    }

    pub fn remove_attribute_by_node_id(&mut self, node_id: usize, name: &str) {
        let mut mutator = self.html_doc.mutate();
        mutator.clear_attribute(node_id, qual(name));
        drop(mutator);
        self.needs_paint = true;
    }

    pub fn add_class_by_id(&mut self, id: &str, class: &str) {
        let nid = self.html_doc.get_element_by_id(id);
        if let Some(node_id) = nid {
            self.add_class_by_node_id(node_id, class);
        }
    }

    pub fn add_class_by_node_id(&mut self, node_id: usize, class: &str) {
        let existing = self.html_doc
            .get_node(node_id)
            .and_then(|n| n.attr(local_name!("class")))
            .map(|s| s.to_string())
            .unwrap_or_default();

        let mut classes: Vec<&str> = existing.split_whitespace().collect();
        if classes.iter().any(|&c| c == class) {
            return;
        }
        classes.push(class);
        let new_value = classes.join(" ");

        let mut mutator = self.html_doc.mutate();
        mutator.set_attribute(node_id, qual("class"), &new_value);
        drop(mutator);
        self.needs_paint = true;
    }

    pub fn remove_class_by_id(&mut self, id: &str, class: &str) {
        let nid = self.html_doc.get_element_by_id(id);
        if let Some(node_id) = nid {
            self.remove_class_by_node_id(node_id, class);
        }
    }

    pub fn remove_class_by_node_id(&mut self, node_id: usize, class: &str) {
        let existing = self.html_doc
            .get_node(node_id)
            .and_then(|n| n.attr(local_name!("class")))
            .map(|s| s.to_string())
            .unwrap_or_default();

        let mut classes: Vec<&str> = existing.split_whitespace().collect();
        classes.retain(|&c| c != class);
        let new_value = classes.join(" ");

        let mut mutator = self.html_doc.mutate();
        mutator.set_attribute(node_id, qual("class"), &new_value);
        drop(mutator);
        self.needs_paint = true;
    }

    // ── Element creation / removal ────────────────────────────────────────

    pub fn create_element_by_id(&mut self, parent_id: &str, tag: &str, attrs: &[(String, String)]) -> Option<usize> {
        let nid = self.html_doc.get_element_by_id(parent_id)?;
        Some(self.create_element_by_node_id(nid, tag, attrs))
    }

    pub fn create_element_by_node_id(&mut self, parent_node_id: usize, tag: &str, attrs: &[(String, String)]) -> usize {
        let blitz_attrs: Vec<blitz_dom::Attribute> = attrs
            .iter()
            .map(|(name, value)| blitz_dom::Attribute {
                name: qual(name),
                value: value.clone(),
            })
            .collect();

        let mut mutator = self.html_doc.mutate();
        let node_id = mutator.create_element(qual(tag), blitz_attrs);
        mutator.append_children(parent_node_id, &[node_id]);
        drop(mutator);
        self.needs_paint = true;
        node_id
    }

    pub fn remove_node_by_id(&mut self, id: &str) {
        if let Some(node_id) = self.html_doc.get_element_by_id(id) {
            self.remove_node_by_node_id(node_id);
        }
    }

    pub fn remove_node_by_node_id(&mut self, node_id: usize) {
        let mut mutator = self.html_doc.mutate();
        mutator.remove_and_drop_node(node_id);
        drop(mutator);
        self.needs_paint = true;
    }
}
