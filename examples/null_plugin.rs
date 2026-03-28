#![allow(non_snake_case)]

#[unsafe(no_mangle)]
pub unsafe extern "C" fn OfxGetNumberOfPlugins() -> ::core::ffi::c_int {
    return 1 as ::core::ffi::c_int;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn OfxGetPlugin(_: ::core::ffi::c_int) -> *mut ::core::ffi::c_void {
    return std::ptr::null_mut();
}
