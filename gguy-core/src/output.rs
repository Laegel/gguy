#[derive(Clone)]
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

#[derive(Clone, Debug)]
pub struct GpuTextureOutput {
    pub vk_image: u64,
    pub width: u32,
    pub height: u32,
}

impl GpuTextureOutput {
    pub fn new(vk_image: u64, width: u32, height: u32) -> Self {
        Self {
            vk_image,
            width,
            height,
        }
    }
}
