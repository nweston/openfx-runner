#![allow(non_snake_case)]

#[unsafe(no_mangle)]
pub unsafe extern "C" fn OfxGetNumberOfPlugins() -> ::core::ffi::c_int {
    return 1 as ::core::ffi::c_int;
}
