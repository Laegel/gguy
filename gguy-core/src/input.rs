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
    },
    PointerUp {
        coords: PointerCoords,
        button: PointerButton,
    },
    PointerMove {
        coords: PointerCoords,
    },
    Scroll {
        delta_x: f64,
        delta_y: f64,
    },
    KeyDown {
        key: String,
    },
    KeyUp {
        key: String,
    },
    TextInput {
        text: String,
    },
}
