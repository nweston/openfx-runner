use crate::{Image, ImageFormat, ImagePixels, Pixel, PixelStorage, Rect};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLBuffer, MTLCommandQueue, MTLCopyAllDevices, MTLDevice};
use std::ffi::c_void;

#[derive(Clone, Debug)]
pub struct GpuContext {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
}

impl GpuContext {
    pub fn new() -> Self {
        let device = MTLCopyAllDevices().firstObject().unwrap();
        let queue = device.newCommandQueue().unwrap();

        Self { device, queue }
    }

    pub fn queue_ptr(&self) -> *const c_void {
        Retained::<ProtocolObject<_>>::as_ptr(&self.queue) as _
    }

    pub fn empty_storage(
        &self,
        format: ImageFormat,
        pixel_count: usize,
    ) -> Box<dyn PixelStorage> {
        Box::new(MetalStorage::empty(format, pixel_count, &self.device))
    }

    pub fn storage_from_image(&self, image: &Image) -> Box<dyn PixelStorage> {
        Box::new(MetalStorage::from_image(image, &self.device))
    }
}

#[derive(Debug)]
struct MetalStorage {
    format: ImageFormat,
    buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    pixel_count: usize,
}

#[cfg(feature = "metal")]
impl MetalStorage {
    fn empty(
        format: ImageFormat,
        pixel_count: usize,
        metal_device: &Retained<ProtocolObject<dyn MTLDevice>>,
    ) -> Self {
        let buffer = metal_device
            .newBufferWithLength_options(
                pixel_count * format.bytes_per_pixel(),
                objc2_metal::MTLResourceOptions::StorageModeShared,
            )
            .unwrap();
        Self {
            format,
            buffer,
            pixel_count,
        }
    }

    fn from_image(
        image: &Image,
        metal_device: &Retained<ProtocolObject<dyn MTLDevice>>,
    ) -> Self {
        let pixel_size = image.data.format().bytes_per_pixel();
        let pixel_count = image.stride * image.bounds.height();
        let byte_count = pixel_size * pixel_count;

        let buffer = metal_device
            .newBufferWithLength_options(
                byte_count,
                objc2_metal::MTLResourceOptions::StorageModeShared,
            )
            .unwrap();
        unsafe {
            std::ptr::copy_nonoverlapping(
                image.data.as_ptr() as *const u8,
                buffer.contents().as_ptr() as *mut u8,
                byte_count,
            );
        }

        Self {
            format: image.data.format(),
            buffer,
            pixel_count,
        }
    }
}

// NOTE: MTLBuffer is not Send so we need this unsafe hack.
unsafe impl Send for MetalStorage {}

#[cfg(feature = "metal")]
impl PixelStorage for MetalStorage {
    fn as_ptr(&self) -> *const c_void {
        Retained::<ProtocolObject<_>>::as_ptr(&self.buffer) as _
    }

    fn as_mut_ptr(&mut self) -> *mut c_void {
        Retained::<ProtocolObject<_>>::as_ptr(&self.buffer) as _
    }

    unsafe fn offset_ptr(&self, _byte_offset: isize) -> *const c_void {
        panic!("Can't offset into Metal buffers.");
    }

    fn format(&self) -> ImageFormat {
        self.format
    }

    fn as_pixels(&self) -> ImagePixels {
        let ptr = self.buffer.contents().as_ptr();
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
