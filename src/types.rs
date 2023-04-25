// Types defined by the OFX API
#![allow(non_snake_case)]

use std::ffi::{c_char, c_double, c_int, c_uint, c_void};

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

// TODO: test that i32 and c_int are the same size
#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum OfxStatus {
    OK = 0,
    Failed = 1,
    ErrFatal = 2,
    ErrUnknown = 3,
    ErrMissingHostFeature = 4,
    ErrUnsupported = 5,
    ErrExists = 6,
    ErrFormat = 7,
    ErrMemory = 8,
    ErrBadHandle = 9,
    ErrBadIndex = 10,
    ErrValue = 11,
    ReplyYes = 12,
    ReplyNo = 13,
    ReplyDefault = 14,
    ErrImageFormat = 1000,
    GLOutOfMemory = 1001,
    GLRenderFailed = 1002,
}

impl OfxStatus {
    pub fn failed(&self) -> bool {
        match self {
            Self::OK | Self::ReplyDefault => false,
            _ => true,
        }
    }

    pub fn succeeded(&self) -> bool {
        !self.failed()
    }
}

pub type OfxTime = c_double;

#[repr(C)]
pub struct OfxRectD {
    pub x1: c_double,
    pub y1: c_double,
    pub x2: c_double,
    pub y2: c_double,
}

#[repr(C)]
pub struct OfxRangeD {
    pub min: c_double,
    pub max: c_double,
}

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
