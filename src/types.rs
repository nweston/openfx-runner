// Types defined by the OFX API
#![allow(non_snake_case)]
pub use openfx_rs::types::{OfxRangeD, OfxRectD, OfxRectI, OfxStatus, OfxTime};
use std::ffi::{c_char, c_int, c_uint, c_void};

macro_rules! handle {
    ($name: ident) => {
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub struct $name(*mut c_void);
        impl From<$name> for *mut c_void {
            fn from(handle: $name) -> Self {
                handle.0
            }
        }
        impl From<*mut c_void> for $name {
            fn from(ptr: *mut c_void) -> Self {
                Self(ptr)
            }
        }
        unsafe impl Send for $name {}
    };
}

handle!(OfxImageClipHandle);
handle!(OfxImageEffectHandle);
handle!(OfxImageMemoryHandle);
handle!(OfxMutexConstHandle);
handle!(OfxMutexHandle);
handle!(OfxParamHandle);
handle!(OfxParamSetHandle);
handle!(OfxPropertySetHandle);

#[repr(C)]
pub struct OfxHost {
    pub host: OfxPropertySetHandle,
    pub fetchSuite:
        extern "C" fn(OfxPropertySetHandle, *const c_char, c_int) -> *const c_void,
}

#[repr(C)]
pub struct OfxPlugin {
    pub pluginApi: *const c_char,
    pub apiVersion: c_int,
    pub pluginIdentifier: *const c_char,
    pub pluginVersionMajor: c_uint,
    pub pluginVersionMinor: c_uint,
    pub setHost: extern "C" fn(*const OfxHost),
    pub mainEntry: extern "C" fn(
        *const c_char,
        *const c_void,
        OfxPropertySetHandle,
        OfxPropertySetHandle,
    ) -> OfxStatus,
}
