use crate::{Image, ImageFormat, PixelStorage};

#[derive(Clone, Debug)]
pub struct GpuContext {}

impl GpuContext {
    pub fn new() -> Self {
        Self {}
    }
}

pub fn empty_gpu_storage(
    _format: ImageFormat,
    _pixel_count: usize,
    _context: &GpuContext,
) -> Box<dyn PixelStorage> {
    panic!("Not compiled with GPU support");
}

pub fn gpu_storage_from_image(
    _image: &Image,
    _context: &GpuContext,
) -> Box<dyn PixelStorage> {
    panic!("Not compiled with GPU support");
}
