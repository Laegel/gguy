pub struct RenderOutput {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl RenderOutput {
    pub fn new(data: Vec<u8>, width: u32, height: u32) -> Self {
        Self { data, width, height }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
