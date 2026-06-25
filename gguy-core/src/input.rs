use keyboard_types::{Code, Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Back,
    Forward,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerCoords {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    PointerDown {
        coords: PointerCoords,
        button: PointerButton,
        modifiers: Modifiers,
    },
    PointerUp {
        coords: PointerCoords,
        button: PointerButton,
        modifiers: Modifiers,
    },
    PointerMove {
        coords: PointerCoords,
        modifiers: Modifiers,
    },
    Scroll {
        delta_x: f64,
        delta_y: f64,
        modifiers: Modifiers,
    },
    KeyDown {
        key: String,
        code: Code,
        modifiers: Modifiers,
    },
    KeyUp {
        key: String,
        code: Code,
        modifiers: Modifiers,
    },
    TextInput {
        text: String,
    },
}
