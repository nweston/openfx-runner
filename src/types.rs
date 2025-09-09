// Types defined by the OFX API
#![allow(non_snake_case)]
pub use openfx_rs::types::{
    OfxHost, OfxPlugin, OfxRangeD, OfxRectD, OfxRectI, OfxStatus, OfxTime,
};
use std::ffi::c_void;

// Define our own handle types which wrap the openfx_rs versions.
//
// This allows us to implement pointer conversions, Hash, and Sync.
macro_rules! handle {
    ($name: ident) => {
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(openfx_rs::types::$name);
        impl From<$name> for *mut c_void {
            fn from(handle: $name) -> Self {
                handle.0 .0 as _
            }
        }
        impl From<*mut c_void> for $name {
            fn from(ptr: *mut c_void) -> Self {
                Self(openfx_rs::types::$name(ptr as _))
            }
        }
        impl From<openfx_rs::types::$name> for $name {
            fn from(h: openfx_rs::types::$name) -> Self {
                Self(h)
            }
        }
        impl From<$name> for openfx_rs::types::$name {
            fn from(handle: $name) -> Self {
                handle.0
            }
        }
        unsafe impl Send for $name {}

        impl std::hash::Hash for $name {
            fn hash<H>(&self, state: &mut H)
            where
                H: std::hash::Hasher,
            {
                self.0 .0.hash(state);
            }
        }
    };
}

handle!(OfxImageClipHandle);
handle!(OfxImageEffectHandle);
handle!(OfxImageMemoryHandle);
handle!(OfxMutexHandle);
handle!(OfxParamHandle);
handle!(OfxParamSetHandle);
handle!(OfxPropertySetHandle);
