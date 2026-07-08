use crate::{Image, ImageFormat, ImagePixels, Pixel, PixelStorage, Rect};
use cust::context::Context as CudaContext;
use cust::memory::DeviceBuffer;
use cust::quick_init;
use std::ffi::c_void;

#[derive(Clone, Debug)]
pub struct GpuContext {
    _context: CudaContext,
}

impl GpuContext {
    pub fn new() -> Self {
        Self {
            _context: quick_init().unwrap(),
        }
    }

    pub fn empty_storage(
        &self,
        format: ImageFormat,
        pixel_count: usize,
    ) -> Box<dyn PixelStorage> {
        Box::new(CudaStorage::empty(format, pixel_count))
    }

    pub fn storage_from_image(&self, image: &Image) -> Box<dyn PixelStorage> {
        Box::new(CudaStorage::from_image(image))
    }
}

#[derive(Debug)]
struct CudaStorage {
    format: ImageFormat,
    buffer: DeviceBuffer<u8>,
    pixel_count: usize,
}

impl CudaStorage {
    fn empty(format: ImageFormat, pixel_count: usize) -> Self {
        let buffer = unsafe {
            DeviceBuffer::<u8>::uninitialized(pixel_count * format.bytes_per_pixel())
                .unwrap()
        };
        Self {
            format,
            buffer,
            pixel_count,
        }
    }

    fn from_image(image: &Image) -> Self {
        let pixel_size = image.data.format().bytes_per_pixel();
        let pixel_count = image.stride * image.bounds.height();
        let byte_count = pixel_size * pixel_count;

        let data = unsafe {
            std::slice::from_raw_parts(image.data.as_ptr() as *const u8, byte_count)
        };

        let buffer = DeviceBuffer::from_slice(data).unwrap();

        Self {
            format: image.data.format(),
            buffer,
            pixel_count,
        }
    }
}

impl PixelStorage for CudaStorage {
    fn as_ptr(&self) -> *const c_void {
        self.buffer.as_device_ptr().as_ptr() as _
    }

    fn as_mut_ptr(&mut self) -> *mut c_void {
        self.buffer.as_device_ptr().as_mut_ptr() as _
    }

    unsafe fn offset_ptr(&self, byte_offset: isize) -> *const c_void {
        unsafe { (self.as_ptr() as *const u8).offset(byte_offset) as _ }
    }

    fn format(&self) -> ImageFormat {
        self.format
    }

    fn as_pixels(&self) -> ImagePixels<'_> {
        let host_data = self.buffer.as_host_vec().unwrap();
        let ptr = host_data.as_ptr();
        unsafe {
            match self.format {
                ImageFormat::Rgba => ImagePixels::Rgba(std::slice::from_raw_parts(
                    ptr as *const Pixel,
                    self.pixel_count,
                )),
                ImageFormat::Alpha => ImagePixels::Alpha(std::slice::from_raw_parts(
                    ptr as *const f32,
                    self.pixel_count,
                )),
            }
        }
    }
}
