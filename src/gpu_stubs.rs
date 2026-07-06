use crate::{Image, ImageFormat, PixelStorage};

#[derive(Clone, Debug)]
pub struct GpuContext {}

impl GpuContext {
    pub fn new() -> Self {
        Self {}
    }

    pub fn empty_storage(
        &self,
        _format: ImageFormat,
        _pixel_count: usize,
    ) -> Box<dyn PixelStorage> {
        panic!("Not compiled with GPU support");
    }

    pub fn storage_from_image(&self, _image: &Image) -> Box<dyn PixelStorage> {
        panic!("Not compiled with GPU support");
    }
}
