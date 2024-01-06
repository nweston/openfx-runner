//! String types for interacting with the OFX API.
//!
//! OFX uses a lot of string constants. These types streamline the
//! process of passing those constants between Rust and C code by
//! checking invariants at construction and then allowing safe,
//! infallible conversion to both String/str and CString/CStr.

use std::ffi::{c_char, CStr, CString};

/// A C-compatible string slice.
///
/// The string is both UTF-8-encoded and null-terminated, so it can
/// safely be converted to both str and CStr without copying. The
/// constructors check these invariants and panic if they don't
/// hold. Since we don't expect this to happen in practice, and we
/// have no useful way to recover if it does, this is a reasonable
/// choice which improves the ergonomics of constructing an OfxStr.
///
/// Note that UTF-8 encoding is not strictly a requirement of the OFX
/// API, but in practice all constants/property names are ASCII.
///
/// OfxStr is not quite analagous to str, because it is contains a
/// reference to its data rather than being a dynamically-sized slice
/// (thus we don't typically use &OfxStr).
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct OfxStr<'a> {
    bytes: &'a [u8],
}

impl<'a> OfxStr<'a> {
    /// Create an OfxStr from a str, or panic if the input is not
    /// null-terminated
    pub const fn from_str(s: &'a str) -> OfxStr<'a> {
        let bytes = s.as_bytes();
        if bytes[bytes.len() - 1] != 0 {
            panic!("OfxStr must be null terminated");
        }
        OfxStr { bytes }
    }

    /// Create an OfxStr from a null-terminated C string, or panic if
    /// the input is not valid UTF-8
    pub fn from_ptr(ptr: *const c_char) -> OfxStr<'a> {
        let cstr = unsafe { CStr::from_ptr(ptr) };
        // Check validity
        match cstr.to_str() {
            Err(e) => panic!("OfxStr must be valid UTF-8: {}", e),
            _ => (),
        }
        OfxStr {
            bytes: cstr.to_bytes_with_nul(),
        }
    }

    pub fn from_cstring(str: &CString) -> OfxStr<'a> {
        Self::from_ptr(str.as_c_str().as_ptr())
    }

    /// Convert to a string slice, omitting the null terminator
    pub fn as_str(&'a self) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(&self.bytes[0..self.bytes.len() - 1]) }
    }

    /// Convert to a C string pointer
    pub fn as_ptr(&self) -> *const c_char {
        self.bytes.as_ptr() as *const c_char
    }

    /// Create a CString with a copy of the string contents
    pub fn to_cstring(&self) -> CString {
        unsafe { CStr::from_bytes_with_nul_unchecked(self.bytes) }.into()
    }
}

impl std::fmt::Display for OfxStr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_str_with_null() {
        let s = OfxStr::from_str("test\0");
        // Note that trailing NULL is omitted from result
        assert_eq!(s.as_str(), "test");
    }

    #[test]
    #[should_panic(expected = "OfxStr must be null terminated")]
    fn from_str_without_null() {
        OfxStr::from_str("test");
    }

    #[test]
    fn from_ptr() {}

    #[test]
    #[should_panic(
        expected = "OfxStr must be valid UTF-8: invalid utf-8 sequence of 1 bytes from index 0"
    )]
    fn from_ptr_not_utf8() {
        let data: [u8; 3] = [0xc3, 0x28, 0x0];
        OfxStr::from_ptr(&data as *const u8 as *const c_char);
    }
}
