// Minimal OFX plugin, derived from the Basic example in the OFX API.
//
// Converted to rust by simplifying, manually converting to C99,
// transpiling with c2rust, and fixing some compilation errors.
#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unsafe_op_in_unsafe_fn,
    unused_assignments,
    unused_mut
)]
pub struct OfxPropertySetStruct {
    _unused: [u8; 0],
}
pub struct OfxImageEffectStruct {
    _unused: [u8; 0],
}
pub struct OfxImageClipStruct {
    _unused: [u8; 0],
}
pub struct OfxImageMemoryStruct {
    _unused: [u8; 0],
}
pub struct OfxParamSetStruct {
    _unused: [u8; 0],
}
pub struct OfxParamStruct {
    _unused: [u8; 0],
}
unsafe extern "C" {
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
}
pub type size_t = usize;
pub type OfxPropertySetHandle = *mut OfxPropertySetStruct;
pub type OfxStatus = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OfxHost {
    pub host: OfxPropertySetHandle,
    pub fetchSuite: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
        ) -> *const ::core::ffi::c_void,
    >,
}
pub type OfxPluginEntryPoint = unsafe extern "C" fn(
    *const ::core::ffi::c_char,
    *const ::core::ffi::c_void,
    OfxPropertySetHandle,
    OfxPropertySetHandle,
) -> OfxStatus;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OfxPlugin {
    pub pluginApi: *const ::core::ffi::c_char,
    pub apiVersion: ::core::ffi::c_int,
    pub pluginIdentifier: *const ::core::ffi::c_char,
    pub pluginVersionMajor: ::core::ffi::c_uint,
    pub pluginVersionMinor: ::core::ffi::c_uint,
    pub setHost: Option<unsafe extern "C" fn(*mut OfxHost) -> ()>,
    pub mainEntry: Option<OfxPluginEntryPoint>,
}
pub type OfxImageEffectHandle = *mut OfxImageEffectStruct;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OfxPropertySuiteV1 {
    pub propSetPointer: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_void,
        ) -> OfxStatus,
    >,
    pub propSetString: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *const ::core::ffi::c_char,
        ) -> OfxStatus,
    >,
    pub propSetDouble: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            ::core::ffi::c_double,
        ) -> OfxStatus,
    >,
    pub propSetInt: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            ::core::ffi::c_int,
        ) -> OfxStatus,
    >,
    pub propSetPointerN: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *const *mut ::core::ffi::c_void,
        ) -> OfxStatus,
    >,
    pub propSetStringN: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *const *const ::core::ffi::c_char,
        ) -> OfxStatus,
    >,
    pub propSetDoubleN: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *const ::core::ffi::c_double,
        ) -> OfxStatus,
    >,
    pub propSetIntN: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *const ::core::ffi::c_int,
        ) -> OfxStatus,
    >,
    pub propGetPointer: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *mut *mut ::core::ffi::c_void,
        ) -> OfxStatus,
    >,
    pub propGetString: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *mut *mut ::core::ffi::c_char,
        ) -> OfxStatus,
    >,
    pub propGetDouble: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_double,
        ) -> OfxStatus,
    >,
    pub propGetInt: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_int,
        ) -> OfxStatus,
    >,
    pub propGetPointerN: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *mut *mut ::core::ffi::c_void,
        ) -> OfxStatus,
    >,
    pub propGetStringN: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *mut *mut ::core::ffi::c_char,
        ) -> OfxStatus,
    >,
    pub propGetDoubleN: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_double,
        ) -> OfxStatus,
    >,
    pub propGetIntN: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_int,
        ) -> OfxStatus,
    >,
    pub propReset: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
        ) -> OfxStatus,
    >,
    pub propGetDimension: Option<
        unsafe extern "C" fn(
            OfxPropertySetHandle,
            *const ::core::ffi::c_char,
            *mut ::core::ffi::c_int,
        ) -> OfxStatus,
    >,
}
pub type OfxImageClipHandle = *mut OfxImageClipStruct;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OfxImageEffectSuiteV1 {
    pub getPropertySet: Option<
        unsafe extern "C" fn(
            OfxImageEffectHandle,
            *mut OfxPropertySetHandle,
        ) -> OfxStatus,
    >,
    pub getParamSet: Option<
        unsafe extern "C" fn(OfxImageEffectHandle, *mut OfxParamSetHandle) -> OfxStatus,
    >,
    pub clipDefine: Option<
        unsafe extern "C" fn(
            OfxImageEffectHandle,
            *const ::core::ffi::c_char,
            *mut OfxPropertySetHandle,
        ) -> OfxStatus,
    >,
    pub clipGetHandle: Option<
        unsafe extern "C" fn(
            OfxImageEffectHandle,
            *const ::core::ffi::c_char,
            *mut OfxImageClipHandle,
            *mut OfxPropertySetHandle,
        ) -> OfxStatus,
    >,
    pub clipGetPropertySet: Option<
        unsafe extern "C" fn(OfxImageClipHandle, *mut OfxPropertySetHandle) -> OfxStatus,
    >,
    pub clipGetImage: Option<
        unsafe extern "C" fn(
            OfxImageClipHandle,
            OfxTime,
            *const OfxRectD,
            *mut OfxPropertySetHandle,
        ) -> OfxStatus,
    >,
    pub clipReleaseImage: Option<unsafe extern "C" fn(OfxPropertySetHandle) -> OfxStatus>,
    pub clipGetRegionOfDefinition: Option<
        unsafe extern "C" fn(OfxImageClipHandle, OfxTime, *mut OfxRectD) -> OfxStatus,
    >,
    pub abort: Option<unsafe extern "C" fn(OfxImageEffectHandle) -> ::core::ffi::c_int>,
    pub imageMemoryAlloc: Option<
        unsafe extern "C" fn(
            OfxImageEffectHandle,
            size_t,
            *mut OfxImageMemoryHandle,
        ) -> OfxStatus,
    >,
    pub imageMemoryFree: Option<unsafe extern "C" fn(OfxImageMemoryHandle) -> OfxStatus>,
    pub imageMemoryLock: Option<
        unsafe extern "C" fn(
            OfxImageMemoryHandle,
            *mut *mut ::core::ffi::c_void,
        ) -> OfxStatus,
    >,
    pub imageMemoryUnlock:
        Option<unsafe extern "C" fn(OfxImageMemoryHandle) -> OfxStatus>,
}
pub type OfxImageMemoryHandle = *mut OfxImageMemoryStruct;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OfxRectD {
    pub x1: ::core::ffi::c_double,
    pub y1: ::core::ffi::c_double,
    pub x2: ::core::ffi::c_double,
    pub y2: ::core::ffi::c_double,
}
pub type OfxTime = ::core::ffi::c_double;
pub type OfxParamSetHandle = *mut OfxParamSetStruct;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MyInstanceData {
    pub isGeneralEffect: bool,
    pub sourceClip: OfxImageClipHandle,
    pub maskClip: OfxImageClipHandle,
    pub outputClip: OfxImageClipHandle,
    pub scaleParam: OfxParamHandle,
}
pub type OfxParamHandle = *mut OfxParamStruct;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OfxRectI {
    pub x1: ::core::ffi::c_int,
    pub y1: ::core::ffi::c_int,
    pub x2: ::core::ffi::c_int,
    pub y2: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OfxRGBAColourF {
    pub r: ::core::ffi::c_float,
    pub g: ::core::ffi::c_float,
    pub b: ::core::ffi::c_float,
    pub a: ::core::ffi::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OfxParameterSuiteV1 {
    pub paramDefine: Option<
        unsafe extern "C" fn(
            OfxParamSetHandle,
            *const ::core::ffi::c_char,
            *const ::core::ffi::c_char,
            *mut OfxPropertySetHandle,
        ) -> OfxStatus,
    >,
    pub paramGetHandle: Option<
        unsafe extern "C" fn(
            OfxParamSetHandle,
            *const ::core::ffi::c_char,
            *mut OfxParamHandle,
            *mut OfxPropertySetHandle,
        ) -> OfxStatus,
    >,
    pub paramSetGetPropertySet: Option<
        unsafe extern "C" fn(OfxParamSetHandle, *mut OfxPropertySetHandle) -> OfxStatus,
    >,
    pub paramGetPropertySet: Option<
        unsafe extern "C" fn(OfxParamHandle, *mut OfxPropertySetHandle) -> OfxStatus,
    >,
    pub paramGetValue: Option<unsafe extern "C" fn(OfxParamHandle, ...) -> OfxStatus>,
    pub paramGetValueAtTime:
        Option<unsafe extern "C" fn(OfxParamHandle, OfxTime, ...) -> OfxStatus>,
    pub paramGetDerivative:
        Option<unsafe extern "C" fn(OfxParamHandle, OfxTime, ...) -> OfxStatus>,
    pub paramGetIntegral:
        Option<unsafe extern "C" fn(OfxParamHandle, OfxTime, OfxTime, ...) -> OfxStatus>,
    pub paramSetValue: Option<unsafe extern "C" fn(OfxParamHandle, ...) -> OfxStatus>,
    pub paramSetValueAtTime:
        Option<unsafe extern "C" fn(OfxParamHandle, OfxTime, ...) -> OfxStatus>,
    pub paramGetNumKeys: Option<
        unsafe extern "C" fn(OfxParamHandle, *mut ::core::ffi::c_uint) -> OfxStatus,
    >,
    pub paramGetKeyTime: Option<
        unsafe extern "C" fn(
            OfxParamHandle,
            ::core::ffi::c_uint,
            *mut OfxTime,
        ) -> OfxStatus,
    >,
    pub paramGetKeyIndex: Option<
        unsafe extern "C" fn(
            OfxParamHandle,
            OfxTime,
            ::core::ffi::c_int,
            *mut ::core::ffi::c_int,
        ) -> OfxStatus,
    >,
    pub paramDeleteKey:
        Option<unsafe extern "C" fn(OfxParamHandle, OfxTime) -> OfxStatus>,
    pub paramDeleteAllKeys: Option<unsafe extern "C" fn(OfxParamHandle) -> OfxStatus>,
    pub paramCopy: Option<
        unsafe extern "C" fn(
            OfxParamHandle,
            OfxParamHandle,
            OfxTime,
            *const OfxRangeD,
        ) -> OfxStatus,
    >,
    pub paramEditBegin: Option<
        unsafe extern "C" fn(OfxParamSetHandle, *const ::core::ffi::c_char) -> OfxStatus,
    >,
    pub paramEditEnd: Option<unsafe extern "C" fn(OfxParamSetHandle) -> OfxStatus>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct OfxRangeD {
    pub min: ::core::ffi::c_double,
    pub max: ::core::ffi::c_double,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 111] = unsafe {
    ::core::mem::transmute::<
        [u8; 111],
        [::core::ffi::c_char; 111],
    >(
        *b"void defineScaleParam(OfxParamSetHandle, const char *, const char *, const char *, const char *, const char *)\0",
    )
};
pub const kOfxActionDescribe: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"OfxActionDescribe\0")
};
pub const kOfxActionCreateInstance: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(
        *b"OfxActionCreateInstance\0",
    )
};
pub const kOfxActionDestroyInstance: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
        *b"OfxActionDestroyInstance\0",
    )
};
pub const kOfxPropTime: [::core::ffi::c_char; 12] = unsafe {
    ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"OfxPropTime\0")
};
pub const kOfxPropInstanceData: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(
        *b"OfxPropInstanceData\0",
    )
};
pub const kOfxPropLabel: [::core::ffi::c_char; 13] = unsafe {
    ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"OfxPropLabel\0")
};
pub const kOfxBitDepthByte: [::core::ffi::c_char; 16] = unsafe {
    ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(*b"OfxBitDepthByte\0")
};
pub const kOfxBitDepthShort: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"OfxBitDepthShort\0")
};
pub const kOfxBitDepthFloat: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"OfxBitDepthFloat\0")
};
pub const kOfxStatOK: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const kOfxStatFailed: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const kOfxStatErrMissingHostFeature: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const kOfxStatReplyDefault: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn ofxuFetchHostSuites() -> OfxStatus {
    unsafe {
        if gHost.is_null() {
            return kOfxStatErrMissingHostFeature;
        }
        gEffectHost = (*gHost).fetchSuite.expect("non-null function pointer")(
            (*gHost).host,
            kOfxImageEffectSuite.as_ptr(),
            1 as ::core::ffi::c_int,
        ) as *mut OfxImageEffectSuiteV1;
        gPropHost = (*gHost).fetchSuite.expect("non-null function pointer")(
            (*gHost).host,
            kOfxPropertySuite.as_ptr(),
            1 as ::core::ffi::c_int,
        ) as *mut OfxPropertySuiteV1;
        gParamHost = (*gHost).fetchSuite.expect("non-null function pointer")(
            (*gHost).host,
            kOfxParameterSuite.as_ptr(),
            1 as ::core::ffi::c_int,
        ) as *mut OfxParameterSuiteV1;
        if gEffectHost.is_null() || gPropHost.is_null() || gParamHost.is_null() {
            return kOfxStatErrMissingHostFeature;
        }
        return kOfxStatOK;
    }
}
#[inline]
unsafe extern "C" fn ofxuIsClipConnected(
    mut pluginInstance: OfxImageEffectHandle,
    mut clipName: *const ::core::ffi::c_char,
) -> bool {
    let mut connected: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut clipHandle: OfxImageClipHandle =
        ::core::ptr::null_mut::<OfxImageClipStruct>();
    let mut props: OfxPropertySetHandle = ::core::ptr::null_mut::<OfxPropertySetStruct>();
    unsafe {
        (*gEffectHost)
            .clipGetHandle
            .expect("non-null function pointer")(
            pluginInstance,
            clipName,
            &raw mut clipHandle,
            &raw mut props,
        );
        if !props.is_null() {
            (*gPropHost).propGetInt.expect("non-null function pointer")(
                props,
                kOfxImageClipPropConnected.as_ptr(),
                0 as ::core::ffi::c_int,
                &raw mut connected,
            );
        }
        return connected != 0 as ::core::ffi::c_int;
    }
}
#[inline]
unsafe extern "C" fn ofxuGetImageData(
    mut imageHandle: OfxPropertySetHandle,
) -> *mut ::core::ffi::c_void {
    unsafe {
        let mut r: *mut ::core::ffi::c_void = NULL;
        (*gPropHost)
            .propGetPointer
            .expect("non-null function pointer")(
            imageHandle,
            kOfxImagePropData.as_ptr(),
            0 as ::core::ffi::c_int,
            &raw mut r,
        );
        return r;
    }
}
#[inline]
unsafe extern "C" fn ofxuGetImageBounds(
    mut imageHandle: OfxPropertySetHandle,
) -> OfxRectI {
    let mut r: OfxRectI = OfxRectI {
        x1: 0 as ::core::ffi::c_int,
        y1: 0 as ::core::ffi::c_int,
        x2: 0 as ::core::ffi::c_int,
        y2: 0 as ::core::ffi::c_int,
    };
    unsafe {
        (*gPropHost).propGetIntN.expect("non-null function pointer")(
            imageHandle,
            kOfxImagePropBounds.as_ptr(),
            4 as ::core::ffi::c_int,
            &raw mut r.x1,
        );
    }
    return r;
}
#[inline]
unsafe extern "C" fn ofxuGetImageRowBytes(
    mut imageHandle: OfxPropertySetHandle,
) -> ::core::ffi::c_int {
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    (*gPropHost).propGetInt.expect("non-null function pointer")(
        imageHandle,
        kOfxImagePropRowBytes.as_ptr(),
        0 as ::core::ffi::c_int,
        &raw mut r,
    );
    return r;
}
#[inline]
unsafe extern "C" fn ofxuMapPixelDepth(
    mut bitString: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if strcmp(bitString, kOfxBitDepthByte.as_ptr()) == 0 as ::core::ffi::c_int {
        return 8 as ::core::ffi::c_int;
    } else if strcmp(bitString, kOfxBitDepthShort.as_ptr()) == 0 as ::core::ffi::c_int {
        return 16 as ::core::ffi::c_int;
    } else if strcmp(bitString, kOfxBitDepthFloat.as_ptr()) == 0 as ::core::ffi::c_int {
        return 32 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn ofxuGetImagePixelDepth(
    mut imageHandle: OfxPropertySetHandle,
    mut isUnMapped: bool,
) -> ::core::ffi::c_int {
    let mut bitString: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    if isUnMapped {
        (*gPropHost)
            .propGetString
            .expect("non-null function pointer")(
            imageHandle,
            kOfxImageClipPropUnmappedPixelDepth.as_ptr(),
            0 as ::core::ffi::c_int,
            &raw mut bitString,
        );
    } else {
        (*gPropHost)
            .propGetString
            .expect("non-null function pointer")(
            imageHandle,
            kOfxImageEffectPropPixelDepth.as_ptr(),
            0 as ::core::ffi::c_int,
            &raw mut bitString,
        );
    }
    return if !bitString.is_null() {
        ofxuMapPixelDepth(bitString)
    } else {
        0 as ::core::ffi::c_int
    };
}
#[inline]
unsafe extern "C" fn ofxuGetImagePixelsAreRGBA(
    mut imageHandle: OfxPropertySetHandle,
    mut unmapped: bool,
) -> bool {
    let mut v: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if unmapped {
        (*gPropHost)
            .propGetString
            .expect("non-null function pointer")(
            imageHandle,
            kOfxImageClipPropUnmappedComponents.as_ptr(),
            0 as ::core::ffi::c_int,
            &raw mut v,
        );
    } else {
        (*gPropHost)
            .propGetString
            .expect("non-null function pointer")(
            imageHandle,
            kOfxImageEffectPropComponents.as_ptr(),
            0 as ::core::ffi::c_int,
            &raw mut v,
        );
    }
    return if !v.is_null() {
        (strcmp(v, kOfxImageComponentAlpha.as_ptr()) != 0 as ::core::ffi::c_int)
            as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    } != 0;
}
#[inline]
unsafe extern "C" fn ofxuGetClipPixelDepth(
    mut clipHandle: OfxImageClipHandle,
    mut unmapped: bool,
) -> ::core::ffi::c_int {
    let mut props: OfxPropertySetHandle = ::core::ptr::null_mut::<OfxPropertySetStruct>();
    (*gEffectHost)
        .clipGetPropertySet
        .expect("non-null function pointer")(clipHandle, &raw mut props);
    return if !props.is_null() {
        ofxuGetImagePixelDepth(props, unmapped)
    } else {
        0 as ::core::ffi::c_int
    };
}
#[inline]
unsafe extern "C" fn ofxuGetClipPixelsAreRGBA(
    mut clipHandle: OfxImageClipHandle,
    mut unmapped: bool,
) -> bool {
    let mut props: OfxPropertySetHandle = ::core::ptr::null_mut::<OfxPropertySetStruct>();
    (*gEffectHost)
        .clipGetPropertySet
        .expect("non-null function pointer")(clipHandle, &raw mut props);
    return if !props.is_null() {
        ofxuGetImagePixelsAreRGBA(props, unmapped) as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    } != 0;
}
#[inline]
unsafe extern "C" fn ofxuClipGetFormat(
    mut clipHandle: OfxImageClipHandle,
    mut bitDepth: *mut ::core::ffi::c_int,
    mut isRGBA: *mut bool,
    mut unmapped: bool,
) {
    *bitDepth = ofxuGetClipPixelDepth(clipHandle, unmapped);
    *isRGBA = ofxuGetClipPixelsAreRGBA(clipHandle, unmapped);
}
#[inline]
unsafe extern "C" fn ofxuGetImage(
    mut clip: OfxImageClipHandle,
    mut time: OfxTime,
    mut rowBytes: *mut ::core::ffi::c_int,
    mut bitDepth: *mut ::core::ffi::c_int,
    mut isAlpha: *mut bool,
    mut rect: *mut OfxRectI,
    mut data: *mut *mut ::core::ffi::c_void,
) -> OfxPropertySetHandle {
    let mut imageProps: OfxPropertySetHandle =
        ::core::ptr::null_mut::<OfxPropertySetStruct>();
    if (*gEffectHost)
        .clipGetImage
        .expect("non-null function pointer")(
        clip,
        time,
        ::core::ptr::null::<OfxRectD>(),
        &raw mut imageProps,
    ) == kOfxStatOK
    {
        *rowBytes = ofxuGetImageRowBytes(imageProps);
        *bitDepth = ofxuGetImagePixelDepth(imageProps, false_0 != 0);
        *isAlpha = !ofxuGetImagePixelsAreRGBA(imageProps, false_0 != 0);
        *rect = ofxuGetImageBounds(imageProps);
        *data = ofxuGetImageData(imageProps);
        if (*data).is_null() {
            (*gEffectHost)
                .clipReleaseImage
                .expect("non-null function pointer")(imageProps);
            imageProps = ::core::ptr::null_mut::<OfxPropertySetStruct>();
        }
    } else {
        *rowBytes = 0 as ::core::ffi::c_int;
        *bitDepth = 0 as ::core::ffi::c_int;
        *isAlpha = false_0 != 0;
        (*rect).y2 = 0 as ::core::ffi::c_int;
        (*rect).y1 = (*rect).y2;
        (*rect).x2 = (*rect).y1;
        (*rect).x1 = (*rect).x2;
        *data = NULL;
    }
    return imageProps;
}
#[unsafe(no_mangle)]
pub static mut gHost: *mut OfxHost = ::core::ptr::null::<OfxHost>() as *mut OfxHost;
#[unsafe(no_mangle)]
pub static mut gEffectHost: *mut OfxImageEffectSuiteV1 =
    ::core::ptr::null::<OfxImageEffectSuiteV1>() as *mut OfxImageEffectSuiteV1;
#[unsafe(no_mangle)]
pub static mut gPropHost: *mut OfxPropertySuiteV1 =
    ::core::ptr::null::<OfxPropertySuiteV1>() as *mut OfxPropertySuiteV1;
#[unsafe(no_mangle)]
pub static mut gParamHost: *mut OfxParameterSuiteV1 =
    ::core::ptr::null::<OfxParameterSuiteV1>() as *mut OfxParameterSuiteV1;
#[unsafe(no_mangle)]
pub static mut gHostSupportsMultipleBitDepths: ::core::ffi::c_int = false_0;
unsafe extern "C" fn getMyInstanceData(
    mut effect: OfxImageEffectHandle,
) -> *mut MyInstanceData {
    let mut effectProps: OfxPropertySetHandle =
        ::core::ptr::null_mut::<OfxPropertySetStruct>();
    (*gEffectHost)
        .getPropertySet
        .expect("non-null function pointer")(effect, &raw mut effectProps);
    let mut myData: *mut MyInstanceData = ::core::ptr::null_mut::<MyInstanceData>();
    (*gPropHost)
        .propGetPointer
        .expect("non-null function pointer")(
        effectProps,
        kOfxPropInstanceData.as_ptr(),
        0 as ::core::ffi::c_int,
        &raw mut myData as *mut *mut ::core::ffi::c_void,
    );
    return myData;
}
unsafe extern "C" fn createInstance(mut effect: OfxImageEffectHandle) -> OfxStatus {
    let mut effectProps: OfxPropertySetHandle =
        ::core::ptr::null_mut::<OfxPropertySetStruct>();
    (*gEffectHost)
        .getPropertySet
        .expect("non-null function pointer")(effect, &raw mut effectProps);
    let mut paramSet: OfxParamSetHandle = ::core::ptr::null_mut::<OfxParamSetStruct>();
    (*gEffectHost)
        .getParamSet
        .expect("non-null function pointer")(effect, &raw mut paramSet);
    let mut myData: *mut MyInstanceData =
        malloc(::core::mem::size_of::<MyInstanceData>() as size_t) as *mut MyInstanceData;
    let mut context: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*gPropHost)
        .propGetString
        .expect("non-null function pointer")(
        effectProps,
        kOfxImageEffectPropContext.as_ptr(),
        0 as ::core::ffi::c_int,
        &raw mut context,
    );
    (*myData).isGeneralEffect = !context.is_null()
        && strcmp(context, kOfxImageEffectContextGeneral.as_ptr())
            == 0 as ::core::ffi::c_int;
    (*gParamHost)
        .paramGetHandle
        .expect("non-null function pointer")(
        paramSet,
        b"scale\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut (*myData).scaleParam,
        ::core::ptr::null_mut::<OfxPropertySetHandle>(),
    );
    (*gEffectHost)
        .clipGetHandle
        .expect("non-null function pointer")(
        effect,
        kOfxImageEffectSimpleSourceClipName.as_ptr(),
        &raw mut (*myData).sourceClip,
        ::core::ptr::null_mut::<OfxPropertySetHandle>(),
    );
    (*gEffectHost)
        .clipGetHandle
        .expect("non-null function pointer")(
        effect,
        kOfxImageEffectOutputClipName.as_ptr(),
        &raw mut (*myData).outputClip,
        ::core::ptr::null_mut::<OfxPropertySetHandle>(),
    );
    if (*myData).isGeneralEffect {
        (*gEffectHost)
            .clipGetHandle
            .expect("non-null function pointer")(
            effect,
            b"Mask\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut (*myData).maskClip,
            ::core::ptr::null_mut::<OfxPropertySetHandle>(),
        );
    } else {
        (*myData).maskClip = ::core::ptr::null_mut::<OfxImageClipStruct>();
    }
    (*gPropHost)
        .propSetPointer
        .expect("non-null function pointer")(
        effectProps,
        kOfxPropInstanceData.as_ptr(),
        0 as ::core::ffi::c_int,
        myData as *mut ::core::ffi::c_void,
    );
    return kOfxStatOK;
}
unsafe extern "C" fn destroyInstance(mut effect: OfxImageEffectHandle) -> OfxStatus {
    let mut myData: *mut MyInstanceData = getMyInstanceData(effect);
    if !myData.is_null() {
        free(myData as *mut ::core::ffi::c_void);
    }
    return kOfxStatOK;
}
unsafe extern "C" fn getClipPreferences(
    mut effect: OfxImageEffectHandle,
    mut _inArgs: OfxPropertySetHandle,
    mut outArgs: OfxPropertySetHandle,
) -> OfxStatus {
    let mut myData: *mut MyInstanceData = getMyInstanceData(effect);
    let mut bitDepth: ::core::ffi::c_int = 0;
    let mut isRGBA: bool = false;
    ofxuClipGetFormat(
        (*myData).sourceClip,
        &raw mut bitDepth,
        &raw mut isRGBA,
        true_0 != 0,
    );
    let mut bitDepthStr: *const ::core::ffi::c_char =
        if bitDepth == 8 as ::core::ffi::c_int {
            kOfxBitDepthByte.as_ptr()
        } else if bitDepth == 16 as ::core::ffi::c_int {
            kOfxBitDepthShort.as_ptr()
        } else {
            kOfxBitDepthFloat.as_ptr()
        };
    let mut componentStr: *const ::core::ffi::c_char =
        if isRGBA as ::core::ffi::c_int != 0 {
            kOfxImageComponentRGBA.as_ptr()
        } else {
            kOfxImageComponentAlpha.as_ptr()
        };
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        outArgs,
        b"OfxImageClipPropComponents_Output\0" as *const u8 as *const ::core::ffi::c_char,
        0 as ::core::ffi::c_int,
        componentStr,
    );
    if gHostSupportsMultipleBitDepths != 0 {
        (*gPropHost)
            .propSetString
            .expect("non-null function pointer")(
            outArgs,
            b"OfxImageClipPropDepth_Output\0" as *const u8 as *const ::core::ffi::c_char,
            0 as ::core::ffi::c_int,
            bitDepthStr,
        );
    }
    if (*myData).isGeneralEffect {
        if ofxuIsClipConnected(
            effect,
            b"Mask\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            (*gPropHost)
                .propSetString
                .expect("non-null function pointer")(
                outArgs,
                b"OfxImageClipPropComponents_Mask\0" as *const u8
                    as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
                kOfxImageComponentAlpha.as_ptr(),
            );
            if gHostSupportsMultipleBitDepths != 0 {
                (*gPropHost)
                    .propSetString
                    .expect("non-null function pointer")(
                    outArgs,
                    b"OfxImageClipPropDepth_Mask\0" as *const u8
                        as *const ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    bitDepthStr,
                );
            }
        }
    }
    return kOfxStatOK;
}
#[inline]
unsafe extern "C" fn pixelAddressRGBA(
    mut img: *mut OfxRGBAColourF,
    mut rect: OfxRectI,
    mut x: ::core::ffi::c_int,
    mut y: ::core::ffi::c_int,
    mut bytesPerLine: ::core::ffi::c_int,
) -> *mut OfxRGBAColourF {
    if x < rect.x1 || x >= rect.x2 || y < rect.y1 || y >= rect.y2 || img.is_null() {
        return ::core::ptr::null_mut::<OfxRGBAColourF>();
    }
    let mut pix: *mut OfxRGBAColourF = (img as *mut ::core::ffi::c_char)
        .offset(((y - rect.y1) * bytesPerLine) as isize)
        as *mut OfxRGBAColourF;
    pix = pix.offset((x - rect.x1) as isize);
    return pix;
}
#[inline]
unsafe extern "C" fn pixelAddressFloat(
    mut img: *mut ::core::ffi::c_float,
    mut rect: OfxRectI,
    mut x: ::core::ffi::c_int,
    mut y: ::core::ffi::c_int,
    mut bytesPerLine: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_float {
    if x < rect.x1 || x >= rect.x2 || y < rect.y1 || y >= rect.y2 || img.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_float>();
    }
    let mut pix: *mut ::core::ffi::c_float = (img as *mut ::core::ffi::c_char)
        .offset(((y - rect.y1) * bytesPerLine) as isize)
        as *mut ::core::ffi::c_float;
    pix = pix.offset((x - rect.x1) as isize);
    return pix;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ProcessRGBA(
    mut instance: OfxImageEffectHandle,
    mut rScale: ::core::ffi::c_float,
    mut gScale: ::core::ffi::c_float,
    mut bScale: ::core::ffi::c_float,
    mut aScale: ::core::ffi::c_float,
    mut srcV: *mut ::core::ffi::c_void,
    mut srcRect: OfxRectI,
    mut srcBytesPerLine: ::core::ffi::c_int,
    mut dstV: *mut ::core::ffi::c_void,
    mut dstRect: OfxRectI,
    mut dstBytesPerLine: ::core::ffi::c_int,
    mut maskV: *mut ::core::ffi::c_void,
    mut maskRect: OfxRectI,
    mut maskBytesPerLine: ::core::ffi::c_int,
    mut procWindow: OfxRectI,
) {
    let mut src: *mut OfxRGBAColourF = srcV as *mut OfxRGBAColourF;
    let mut dst: *mut OfxRGBAColourF = dstV as *mut OfxRGBAColourF;
    let mut mask: *mut ::core::ffi::c_float = maskV as *mut ::core::ffi::c_float;
    let mut y: ::core::ffi::c_int = procWindow.y1;
    while y < procWindow.y2 {
        if (*gEffectHost).abort.expect("non-null function pointer")(instance) != 0 {
            break;
        }
        let mut dstPix: *mut OfxRGBAColourF =
            pixelAddressRGBA(dst, dstRect, procWindow.x1, y, dstBytesPerLine);
        let mut x: ::core::ffi::c_int = procWindow.x1;
        while x < procWindow.x2 {
            let mut srcPix: *mut OfxRGBAColourF =
                pixelAddressRGBA(src, srcRect, x, y, srcBytesPerLine);
            let mut maskV_0: ::core::ffi::c_float = 1.0f32;
            if !mask.is_null() {
                let mut maskPix: *mut ::core::ffi::c_float =
                    pixelAddressFloat(mask, maskRect, x, y, maskBytesPerLine);
                if !maskPix.is_null() {
                    maskV_0 = *maskPix / 1.0f32;
                } else {
                    maskV_0 = 0.0f32;
                }
                maskPix = maskPix.offset(1);
            }
            let mut sR: ::core::ffi::c_float = (1.0f64
                + (rScale as ::core::ffi::c_double - 1.0f64)
                    * maskV_0 as ::core::ffi::c_double)
                as ::core::ffi::c_float;
            let mut sG: ::core::ffi::c_float = (1.0f64
                + (gScale as ::core::ffi::c_double - 1.0f64)
                    * maskV_0 as ::core::ffi::c_double)
                as ::core::ffi::c_float;
            let mut sB: ::core::ffi::c_float = (1.0f64
                + (bScale as ::core::ffi::c_double - 1.0f64)
                    * maskV_0 as ::core::ffi::c_double)
                as ::core::ffi::c_float;
            let mut sA: ::core::ffi::c_float = (1.0f64
                + (aScale as ::core::ffi::c_double - 1.0f64)
                    * maskV_0 as ::core::ffi::c_double)
                as ::core::ffi::c_float;
            if !srcPix.is_null() {
                (*dstPix).r = (*srcPix).r * sR;
                (*dstPix).g = (*srcPix).g * sG;
                (*dstPix).b = (*srcPix).b * sB;
                (*dstPix).a = (*srcPix).a * sA;
                srcPix = srcPix.offset(1);
            } else {
                (*dstPix).a = 0 as ::core::ffi::c_int as ::core::ffi::c_float;
                (*dstPix).b = (*dstPix).a;
                (*dstPix).g = (*dstPix).b;
                (*dstPix).r = (*dstPix).g;
            }
            dstPix = dstPix.offset(1);
            x += 1;
        }
        y += 1;
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ProcessAlpha(
    mut instance: OfxImageEffectHandle,
    mut scale: ::core::ffi::c_float,
    mut srcV: *mut ::core::ffi::c_void,
    mut srcRect: OfxRectI,
    mut srcBytesPerLine: ::core::ffi::c_int,
    mut dstV: *mut ::core::ffi::c_void,
    mut dstRect: OfxRectI,
    mut dstBytesPerLine: ::core::ffi::c_int,
    mut maskV: *mut ::core::ffi::c_void,
    mut maskRect: OfxRectI,
    mut maskBytesPerLine: ::core::ffi::c_int,
    mut procWindow: OfxRectI,
) {
    let mut src: *mut ::core::ffi::c_float = srcV as *mut ::core::ffi::c_float;
    let mut dst: *mut ::core::ffi::c_float = dstV as *mut ::core::ffi::c_float;
    let mut mask: *mut ::core::ffi::c_float = maskV as *mut ::core::ffi::c_float;
    let mut y: ::core::ffi::c_int = procWindow.y1;
    while y < procWindow.y2 {
        if (*gEffectHost).abort.expect("non-null function pointer")(instance) != 0 {
            break;
        }
        let mut dstPix: *mut ::core::ffi::c_float =
            pixelAddressFloat(dst, dstRect, procWindow.x1, y, dstBytesPerLine);
        let mut x: ::core::ffi::c_int = procWindow.x1;
        while x < procWindow.x2 {
            let mut srcPix: *mut ::core::ffi::c_float =
                pixelAddressFloat(src, srcRect, x, y, srcBytesPerLine);
            let mut maskV_0: ::core::ffi::c_float = 1.0f32;
            if !mask.is_null() {
                let mut maskPix: *mut ::core::ffi::c_float =
                    pixelAddressFloat(mask, maskRect, x, y, maskBytesPerLine);
                if !maskPix.is_null() {
                    maskV_0 = *maskPix / 1.0f32;
                }
            }
            let mut theScale: ::core::ffi::c_float = (1.0f64
                + (scale as ::core::ffi::c_double - 1.0f64)
                    * maskV_0 as ::core::ffi::c_double)
                as ::core::ffi::c_float;
            if !srcPix.is_null() {
                *dstPix = *srcPix * theScale;
                srcPix = srcPix.offset(1);
            } else {
                *dstPix = 0 as ::core::ffi::c_int as ::core::ffi::c_float;
            }
            dstPix = dstPix.offset(1);
            x += 1;
        }
        y += 1;
    }
}
unsafe extern "C" fn render(
    mut instance: OfxImageEffectHandle,
    mut inArgs: OfxPropertySetHandle,
    mut _outArgs: OfxPropertySetHandle,
) -> OfxStatus {
    let mut time: OfxTime = 0.;
    let mut renderWindow: OfxRectI = OfxRectI {
        x1: 0,
        y1: 0,
        x2: 0,
        y2: 0,
    };
    let mut status: OfxStatus = kOfxStatOK;
    (*gPropHost)
        .propGetDouble
        .expect("non-null function pointer")(
        inArgs,
        kOfxPropTime.as_ptr(),
        0 as ::core::ffi::c_int,
        &raw mut time,
    );
    (*gPropHost).propGetIntN.expect("non-null function pointer")(
        inArgs,
        kOfxImageEffectPropRenderWindow.as_ptr(),
        4 as ::core::ffi::c_int,
        &raw mut renderWindow.x1,
    );
    let mut myData: *mut MyInstanceData = getMyInstanceData(instance);
    let mut sourceImg: OfxPropertySetHandle =
        ::core::ptr::null_mut::<OfxPropertySetStruct>();
    let mut outputImg: OfxPropertySetHandle =
        ::core::ptr::null_mut::<OfxPropertySetStruct>();
    let mut maskImg: OfxPropertySetHandle =
        ::core::ptr::null_mut::<OfxPropertySetStruct>();
    let mut srcRowBytes: ::core::ffi::c_int = 0;
    let mut srcBitDepth: ::core::ffi::c_int = 0;
    let mut dstRowBytes: ::core::ffi::c_int = 0;
    let mut dstBitDepth: ::core::ffi::c_int = 0;
    let mut maskRowBytes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut maskBitDepth: ::core::ffi::c_int = 0;
    let mut srcIsAlpha: bool = false;
    let mut dstIsAlpha: bool = false;
    let mut maskIsAlpha: bool = false_0 != 0;
    let mut dstRect: OfxRectI = OfxRectI {
        x1: 0,
        y1: 0,
        x2: 0,
        y2: 0,
    };
    let mut srcRect: OfxRectI = OfxRectI {
        x1: 0,
        y1: 0,
        x2: 0,
        y2: 0,
    };
    let mut maskRect: OfxRectI = OfxRectI {
        x1: 0 as ::core::ffi::c_int,
        y1: 0 as ::core::ffi::c_int,
        x2: 0 as ::core::ffi::c_int,
        y2: 0 as ::core::ffi::c_int,
    };
    let mut src: *mut ::core::ffi::c_void =
        ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut dst: *mut ::core::ffi::c_void =
        ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut mask: *mut ::core::ffi::c_void = NULL;
    sourceImg = ofxuGetImage(
        (*myData).sourceClip,
        time,
        &raw mut srcRowBytes,
        &raw mut srcBitDepth,
        &raw mut srcIsAlpha,
        &raw mut srcRect,
        &raw mut src,
    );
    if sourceImg.is_null() {
        return kOfxStatFailed;
    }
    outputImg = ofxuGetImage(
        (*myData).outputClip,
        time,
        &raw mut dstRowBytes,
        &raw mut dstBitDepth,
        &raw mut dstIsAlpha,
        &raw mut dstRect,
        &raw mut dst,
    );
    if outputImg.is_null() {
        return kOfxStatFailed;
    }
    if (*myData).isGeneralEffect {
        if ofxuIsClipConnected(
            instance,
            b"Mask\0" as *const u8 as *const ::core::ffi::c_char,
        ) {
            maskImg = ofxuGetImage(
                (*myData).maskClip,
                time,
                &raw mut maskRowBytes,
                &raw mut maskBitDepth,
                &raw mut maskIsAlpha,
                &raw mut maskRect,
                &raw mut mask,
            );
            if !maskImg.is_null() {
                if !maskIsAlpha || maskBitDepth != srcBitDepth {
                    return kOfxStatFailed;
                }
            }
        }
    }
    if srcBitDepth != dstBitDepth
        || srcIsAlpha as ::core::ffi::c_int != dstIsAlpha as ::core::ffi::c_int
    {
        return kOfxStatFailed;
    }
    let mut scale: ::core::ffi::c_double = 0.;
    (*gParamHost)
        .paramGetValueAtTime
        .expect("non-null function pointer")(
        (*myData).scaleParam, time, &raw mut scale
    );
    if !dstIsAlpha {
        ProcessRGBA(
            instance,
            scale as ::core::ffi::c_float,
            scale as ::core::ffi::c_float,
            scale as ::core::ffi::c_float,
            scale as ::core::ffi::c_float,
            src,
            srcRect,
            srcRowBytes,
            dst,
            dstRect,
            dstRowBytes,
            mask,
            maskRect,
            maskRowBytes,
            renderWindow,
        );
    } else {
        ProcessAlpha(
            instance,
            scale as ::core::ffi::c_float,
            src,
            srcRect,
            srcRowBytes,
            dst,
            dstRect,
            dstRowBytes,
            mask,
            maskRect,
            maskRowBytes,
            renderWindow,
        );
    }
    if !maskImg.is_null() {
        (*gEffectHost)
            .clipReleaseImage
            .expect("non-null function pointer")(maskImg);
    }
    if !sourceImg.is_null() {
        (*gEffectHost)
            .clipReleaseImage
            .expect("non-null function pointer")(sourceImg);
    }
    if !outputImg.is_null() {
        (*gEffectHost)
            .clipReleaseImage
            .expect("non-null function pointer")(outputImg);
    }
    return status;
}
unsafe extern "C" fn defineScaleParam(
    mut effectParams: OfxParamSetHandle,
    mut name: *const ::core::ffi::c_char,
    mut label: *const ::core::ffi::c_char,
    mut scriptName: *const ::core::ffi::c_char,
    mut hint: *const ::core::ffi::c_char,
    mut parent: *const ::core::ffi::c_char,
) {
    let mut props: OfxPropertySetHandle = ::core::ptr::null_mut::<OfxPropertySetStruct>();
    let mut stat: OfxStatus = 0;
    stat = (*gParamHost)
        .paramDefine
        .expect("non-null function pointer")(
        effectParams,
        kOfxParamTypeDouble.as_ptr(),
        name,
        &raw mut props,
    );
    if stat == 0 as ::core::ffi::c_int {
    } else {
        __assert_fail(
            b"stat == kOfxStatOK\0" as *const u8 as *const ::core::ffi::c_char,
            b"basic.c\0" as *const u8 as *const ::core::ffi::c_char,
            561 as ::core::ffi::c_uint,
            __ASSERT_FUNCTION.as_ptr(),
        );
    };
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        props,
        kOfxParamPropDoubleType.as_ptr(),
        0 as ::core::ffi::c_int,
        kOfxParamDoubleTypeScale.as_ptr(),
    );
    (*gPropHost)
        .propSetDouble
        .expect("non-null function pointer")(
        props,
        kOfxParamPropDefault.as_ptr(),
        0 as ::core::ffi::c_int,
        1.0f64,
    );
    (*gPropHost)
        .propSetDouble
        .expect("non-null function pointer")(
        props,
        kOfxParamPropMin.as_ptr(),
        0 as ::core::ffi::c_int,
        0.0f64,
    );
    (*gPropHost)
        .propSetDouble
        .expect("non-null function pointer")(
        props,
        kOfxParamPropDisplayMin.as_ptr(),
        0 as ::core::ffi::c_int,
        0.0f64,
    );
    (*gPropHost)
        .propSetDouble
        .expect("non-null function pointer")(
        props,
        kOfxParamPropDisplayMax.as_ptr(),
        0 as ::core::ffi::c_int,
        100.0f64,
    );
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        props,
        kOfxParamPropHint.as_ptr(),
        0 as ::core::ffi::c_int,
        hint,
    );
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        props,
        kOfxParamPropScriptName.as_ptr(),
        0 as ::core::ffi::c_int,
        scriptName,
    );
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        props,
        kOfxPropLabel.as_ptr(),
        0 as ::core::ffi::c_int,
        label,
    );
    if !parent.is_null() {
        (*gPropHost)
            .propSetString
            .expect("non-null function pointer")(
            props,
            kOfxParamPropParent.as_ptr(),
            0 as ::core::ffi::c_int,
            parent,
        );
    }
}
unsafe extern "C" fn describeInContext(
    mut effect: OfxImageEffectHandle,
    mut inArgs: OfxPropertySetHandle,
) -> OfxStatus {
    let mut context: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*gPropHost)
        .propGetString
        .expect("non-null function pointer")(
        inArgs,
        kOfxImageEffectPropContext.as_ptr(),
        0 as ::core::ffi::c_int,
        &raw mut context,
    );
    let mut isGeneralContext: bool =
        strcmp(context, kOfxImageEffectContextGeneral.as_ptr())
            == 0 as ::core::ffi::c_int;
    let mut props: OfxPropertySetHandle = ::core::ptr::null_mut::<OfxPropertySetStruct>();
    (*gEffectHost)
        .clipDefine
        .expect("non-null function pointer")(
        effect,
        kOfxImageEffectOutputClipName.as_ptr(),
        &raw mut props,
    );
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        props,
        kOfxImageEffectPropSupportedComponents.as_ptr(),
        0 as ::core::ffi::c_int,
        kOfxImageComponentRGBA.as_ptr(),
    );
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        props,
        kOfxImageEffectPropSupportedComponents.as_ptr(),
        1 as ::core::ffi::c_int,
        kOfxImageComponentAlpha.as_ptr(),
    );
    (*gEffectHost)
        .clipDefine
        .expect("non-null function pointer")(
        effect,
        kOfxImageEffectSimpleSourceClipName.as_ptr(),
        &raw mut props,
    );
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        props,
        kOfxImageEffectPropSupportedComponents.as_ptr(),
        0 as ::core::ffi::c_int,
        kOfxImageComponentRGBA.as_ptr(),
    );
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        props,
        kOfxImageEffectPropSupportedComponents.as_ptr(),
        1 as ::core::ffi::c_int,
        kOfxImageComponentAlpha.as_ptr(),
    );
    if isGeneralContext {
        (*gEffectHost)
            .clipDefine
            .expect("non-null function pointer")(
            effect,
            b"Mask\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut props,
        );
        (*gPropHost)
            .propSetString
            .expect("non-null function pointer")(
            props,
            kOfxImageEffectPropSupportedComponents.as_ptr(),
            0 as ::core::ffi::c_int,
            kOfxImageComponentAlpha.as_ptr(),
        );
        (*gPropHost).propSetInt.expect("non-null function pointer")(
            props,
            kOfxImageClipPropOptional.as_ptr(),
            0 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        );
    }
    let mut paramSet: OfxParamSetHandle = ::core::ptr::null_mut::<OfxParamSetStruct>();
    (*gEffectHost)
        .getParamSet
        .expect("non-null function pointer")(effect, &raw mut paramSet);
    defineScaleParam(
        paramSet,
        b"scale\0" as *const u8 as *const ::core::ffi::c_char,
        b"Scale\0" as *const u8 as *const ::core::ffi::c_char,
        b"scale\0" as *const u8 as *const ::core::ffi::c_char,
        b"Scales all component in the image\0" as *const u8 as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
    );
    (*gParamHost)
        .paramDefine
        .expect("non-null function pointer")(
        paramSet,
        kOfxParamTypePage.as_ptr(),
        b"Main\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut props,
    );
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        props,
        kOfxParamPropPageChild.as_ptr(),
        0 as ::core::ffi::c_int,
        b"scale\0" as *const u8 as *const ::core::ffi::c_char,
    );
    return kOfxStatOK;
}
unsafe extern "C" fn describe(mut effect: OfxImageEffectHandle) -> OfxStatus {
    let mut stat: OfxStatus = 0;
    stat = ofxuFetchHostSuites();
    if stat != kOfxStatOK {
        return stat;
    }
    (*gPropHost).propGetInt.expect("non-null function pointer")(
        (*gHost).host,
        kOfxImageEffectPropSupportsMultipleClipDepths.as_ptr(),
        0 as ::core::ffi::c_int,
        &raw mut gHostSupportsMultipleBitDepths,
    );
    let mut effectProps: OfxPropertySetHandle =
        ::core::ptr::null_mut::<OfxPropertySetStruct>();
    (*gEffectHost)
        .getPropertySet
        .expect("non-null function pointer")(effect, &raw mut effectProps);
    (*gPropHost).propSetInt.expect("non-null function pointer")(
        effectProps,
        kOfxImageEffectPluginPropFieldRenderTwiceAlways.as_ptr(),
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    (*gPropHost).propSetInt.expect("non-null function pointer")(
        effectProps,
        kOfxImageEffectPropSupportsMultipleClipDepths.as_ptr(),
        0 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        effectProps,
        kOfxImageEffectPropSupportedPixelDepths.as_ptr(),
        0 as ::core::ffi::c_int,
        kOfxBitDepthFloat.as_ptr(),
    );
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        effectProps,
        kOfxPropLabel.as_ptr(),
        0 as ::core::ffi::c_int,
        b"OFX Gain Example\0" as *const u8 as *const ::core::ffi::c_char,
    );
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        effectProps,
        kOfxImageEffectPluginPropGrouping.as_ptr(),
        0 as ::core::ffi::c_int,
        b"OFX Example\0" as *const u8 as *const ::core::ffi::c_char,
    );
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        effectProps,
        kOfxImageEffectPropSupportedContexts.as_ptr(),
        0 as ::core::ffi::c_int,
        kOfxImageEffectContextFilter.as_ptr(),
    );
    (*gPropHost)
        .propSetString
        .expect("non-null function pointer")(
        effectProps,
        kOfxImageEffectPropSupportedContexts.as_ptr(),
        1 as ::core::ffi::c_int,
        kOfxImageEffectContextGeneral.as_ptr(),
    );
    return kOfxStatOK;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getSpatialRoD(
    mut effect: OfxImageEffectHandle,
    mut inArgs: OfxPropertySetHandle,
    mut outArgs: OfxPropertySetHandle,
) -> OfxStatus {
    let mut myData: *mut MyInstanceData = getMyInstanceData(effect);
    let mut time: OfxTime = 0.;
    (*gPropHost)
        .propGetDouble
        .expect("non-null function pointer")(
        inArgs,
        kOfxPropTime.as_ptr(),
        0 as ::core::ffi::c_int,
        &raw mut time,
    );
    let mut rod: OfxRectD = OfxRectD {
        x1: 0.,
        y1: 0.,
        x2: 0.,
        y2: 0.,
    };
    (*gEffectHost)
        .clipGetRegionOfDefinition
        .expect("non-null function pointer")((*myData).sourceClip, time, &raw mut rod);
    rod.x1 -= 1.;
    rod.y1 -= 2.;
    rod.x2 += 3.;
    rod.y2 += 4.;
    (*gPropHost)
        .propSetDoubleN
        .expect("non-null function pointer")(
        outArgs,
        kOfxImageEffectPropRegionOfDefinition.as_ptr(),
        4 as ::core::ffi::c_int,
        &raw mut rod.x1,
    );
    return kOfxStatOK;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getSpatialRoI(
    mut effect: OfxImageEffectHandle,
    mut inArgs: OfxPropertySetHandle,
    mut outArgs: OfxPropertySetHandle,
) -> OfxStatus {
    let mut roi: OfxRectD = OfxRectD {
        x1: 0.,
        y1: 0.,
        x2: 0.,
        y2: 0.,
    };
    (*gPropHost)
        .propGetDoubleN
        .expect("non-null function pointer")(
        inArgs,
        kOfxImageEffectPropRegionOfInterest.as_ptr(),
        4 as ::core::ffi::c_int,
        &raw mut roi.x1,
    );

    roi.x1 += 1.;
    roi.y1 += 2.;
    roi.x2 -= 3.;
    roi.y2 -= 4.;

    (*gPropHost)
        .propSetDoubleN
        .expect("non-null function pointer")(
        outArgs,
        b"OfxImageClipPropRoI_Source\0" as *const u8 as *const ::core::ffi::c_char,
        4 as ::core::ffi::c_int,
        &raw mut roi.x1,
    );
    let mut myData: *mut MyInstanceData = getMyInstanceData(effect);
    if (*myData).isGeneralEffect as ::core::ffi::c_int != 0
        && ofxuIsClipConnected(
            effect,
            b"Mask\0" as *const u8 as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int
            != 0
    {
        (*gPropHost)
            .propSetDoubleN
            .expect("non-null function pointer")(
            outArgs,
            b"OfxImageClipPropRoI_Mask\0" as *const u8 as *const ::core::ffi::c_char,
            4 as ::core::ffi::c_int,
            &raw mut roi.x1,
        );
    }
    return kOfxStatOK;
}
unsafe extern "C" fn pluginMain(
    mut action: *const ::core::ffi::c_char,
    mut handle: *const ::core::ffi::c_void,
    mut inArgs: OfxPropertySetHandle,
    mut outArgs: OfxPropertySetHandle,
) -> OfxStatus {
    let mut effect: OfxImageEffectHandle = handle as OfxImageEffectHandle;
    if strcmp(action, kOfxActionDescribe.as_ptr()) == 0 as ::core::ffi::c_int {
        return describe(effect);
    } else if strcmp(action, kOfxImageEffectActionDescribeInContext.as_ptr())
        == 0 as ::core::ffi::c_int
    {
        return describeInContext(effect, inArgs);
    } else if strcmp(action, kOfxActionCreateInstance.as_ptr()) == 0 as ::core::ffi::c_int
    {
        return createInstance(effect);
    } else if strcmp(action, kOfxActionDestroyInstance.as_ptr())
        == 0 as ::core::ffi::c_int
    {
        return destroyInstance(effect);
    } else if strcmp(action, kOfxImageEffectActionRender.as_ptr())
        == 0 as ::core::ffi::c_int
    {
        return render(effect, inArgs, outArgs);
    } else if strcmp(action, kOfxImageEffectActionGetRegionOfDefinition.as_ptr())
        == 0 as ::core::ffi::c_int
    {
        return getSpatialRoD(effect, inArgs, outArgs);
    } else if strcmp(action, kOfxImageEffectActionGetRegionsOfInterest.as_ptr())
        == 0 as ::core::ffi::c_int
    {
        return getSpatialRoI(effect, inArgs, outArgs);
    } else if strcmp(action, kOfxImageEffectActionGetClipPreferences.as_ptr())
        == 0 as ::core::ffi::c_int
    {
        return getClipPreferences(effect, inArgs, outArgs);
    }
    return kOfxStatReplyDefault;
}
unsafe extern "C" fn setHostFunc(mut hostStruct: *mut OfxHost) {
    gHost = hostStruct;
}
static mut basicPlugin: OfxPlugin = OfxPlugin {
    pluginApi: kOfxImageEffectPluginApi.as_ptr(),
    apiVersion: 1 as ::core::ffi::c_int,
    pluginIdentifier: b"uk.co.thefoundry.BasicGainPlugin\0" as *const u8
        as *const ::core::ffi::c_char,
    pluginVersionMajor: 1 as ::core::ffi::c_uint,
    pluginVersionMinor: 0 as ::core::ffi::c_uint,
    setHost: Some(setHostFunc as unsafe extern "C" fn(*mut OfxHost) -> ()),
    mainEntry: Some(
        pluginMain
            as unsafe extern "C" fn(
                *const ::core::ffi::c_char,
                *const ::core::ffi::c_void,
                OfxPropertySetHandle,
                OfxPropertySetHandle,
            ) -> OfxStatus,
    ),
};
#[unsafe(no_mangle)]
pub unsafe extern "C" fn OfxGetPlugin(mut nth: ::core::ffi::c_int) -> *mut OfxPlugin {
    if nth == 0 as ::core::ffi::c_int {
        return &raw mut basicPlugin;
    }
    return ::core::ptr::null_mut::<OfxPlugin>();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn OfxGetNumberOfPlugins() -> ::core::ffi::c_int {
    return 1 as ::core::ffi::c_int;
}
pub const kOfxParameterSuite: [::core::ffi::c_char; 18] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"OfxParameterSuite\0")
};
pub const kOfxParamTypeDouble: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(
        *b"OfxParamTypeDouble\0",
    )
};
pub const kOfxParamTypePage: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"OfxParamTypePage\0")
};
pub const kOfxParamPropScriptName: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(
        *b"OfxParamPropScriptName\0",
    )
};
pub const kOfxParamPropHint: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"OfxParamPropHint\0")
};
pub const kOfxParamPropDefault: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(
        *b"OfxParamPropDefault\0",
    )
};
pub const kOfxParamPropDoubleType: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(
        *b"OfxParamPropDoubleType\0",
    )
};
pub const kOfxParamDoubleTypeScale: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(
        *b"OfxParamDoubleTypeScale\0",
    )
};
pub const kOfxParamPropPageChild: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(
        *b"OfxParamPropPageChild\0",
    )
};
pub const kOfxParamPropParent: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(
        *b"OfxParamPropParent\0",
    )
};
pub const kOfxParamPropMin: [::core::ffi::c_char; 16] = unsafe {
    ::core::mem::transmute::<[u8; 16], [::core::ffi::c_char; 16]>(*b"OfxParamPropMin\0")
};
pub const kOfxParamPropDisplayMin: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(
        *b"OfxParamPropDisplayMin\0",
    )
};
pub const kOfxParamPropDisplayMax: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(
        *b"OfxParamPropDisplayMax\0",
    )
};
pub const kOfxImageEffectPluginApi: [::core::ffi::c_char; 24] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_char; 24]>(
        *b"OfxImageEffectPluginAPI\0",
    )
};
pub const kOfxImageComponentRGBA: [::core::ffi::c_char; 22] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_char; 22]>(
        *b"OfxImageComponentRGBA\0",
    )
};
pub const kOfxImageComponentAlpha: [::core::ffi::c_char; 23] = unsafe {
    ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(
        *b"OfxImageComponentAlpha\0",
    )
};
pub const kOfxImageEffectContextFilter: [::core::ffi::c_char; 28] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_char; 28]>(
        *b"OfxImageEffectContextFilter\0",
    )
};
pub const kOfxImageEffectContextGeneral: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"OfxImageEffectContextGeneral\0",
    )
};
pub const kOfxImageEffectActionGetRegionOfDefinition: [::core::ffi::c_char; 42] = unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_char; 42]>(
        *b"OfxImageEffectActionGetRegionOfDefinition\0",
    )
};
pub const kOfxImageEffectActionGetRegionsOfInterest: [::core::ffi::c_char; 41] = unsafe {
    ::core::mem::transmute::<[u8; 41], [::core::ffi::c_char; 41]>(
        *b"OfxImageEffectActionGetRegionsOfInterest\0",
    )
};
pub const kOfxImageEffectActionGetClipPreferences: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"OfxImageEffectActionGetClipPreferences\0",
    )
};
pub const kOfxImageEffectActionRender: [::core::ffi::c_char; 27] = unsafe {
    ::core::mem::transmute::<[u8; 27], [::core::ffi::c_char; 27]>(
        *b"OfxImageEffectActionRender\0",
    )
};
pub const kOfxImageEffectActionDescribeInContext: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"OfxImageEffectActionDescribeInContext\0",
    )
};
pub const kOfxImageEffectPropSupportedContexts: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"OfxImageEffectPropSupportedContexts\0",
    )
};
pub const kOfxImageEffectPropSupportsMultipleClipDepths: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"OfxImageEffectPropMultipleClipDepths\0",
    )
};
pub const kOfxImageEffectPluginPropGrouping: [::core::ffi::c_char; 33] = unsafe {
    ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
        *b"OfxImageEffectPluginPropGrouping\0",
    )
};
pub const kOfxImageEffectPropContext: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(
        *b"OfxImageEffectPropContext\0",
    )
};
pub const kOfxImageEffectPropPixelDepth: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"OfxImageEffectPropPixelDepth\0",
    )
};
pub const kOfxImageEffectPropComponents: [::core::ffi::c_char; 29] = unsafe {
    ::core::mem::transmute::<[u8; 29], [::core::ffi::c_char; 29]>(
        *b"OfxImageEffectPropComponents\0",
    )
};
pub const kOfxImageClipPropUnmappedPixelDepth: [::core::ffi::c_char; 35] = unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"OfxImageClipPropUnmappedPixelDepth\0",
    )
};
pub const kOfxImageClipPropUnmappedComponents: [::core::ffi::c_char; 35] = unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"OfxImageClipPropUnmappedComponents\0",
    )
};
pub const kOfxImageEffectPropSupportedPixelDepths: [::core::ffi::c_char; 39] = unsafe {
    ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
        *b"OfxImageEffectPropSupportedPixelDepths\0",
    )
};
pub const kOfxImageEffectPropSupportedComponents: [::core::ffi::c_char; 38] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
        *b"OfxImageEffectPropSupportedComponents\0",
    )
};
pub const kOfxImageClipPropOptional: [::core::ffi::c_char; 25] = unsafe {
    ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
        *b"OfxImageClipPropOptional\0",
    )
};
pub const kOfxImageClipPropConnected: [::core::ffi::c_char; 26] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_char; 26]>(
        *b"OfxImageClipPropConnected\0",
    )
};
pub const kOfxImagePropData: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"OfxImagePropData\0")
};
pub const kOfxImagePropBounds: [::core::ffi::c_char; 19] = unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(
        *b"OfxImagePropBounds\0",
    )
};
pub const kOfxImagePropRowBytes: [::core::ffi::c_char; 21] = unsafe {
    ::core::mem::transmute::<[u8; 21], [::core::ffi::c_char; 21]>(
        *b"OfxImagePropRowBytes\0",
    )
};
pub const kOfxImageEffectPluginPropFieldRenderTwiceAlways: [::core::ffi::c_char; 47] = unsafe {
    ::core::mem::transmute::<[u8; 47], [::core::ffi::c_char; 47]>(
        *b"OfxImageEffectPluginPropFieldRenderTwiceAlways\0",
    )
};
pub const kOfxImageEffectPropRegionOfDefinition: [::core::ffi::c_char; 37] = unsafe {
    ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
        *b"OfxImageEffectPropRegionOfDefinition\0",
    )
};
pub const kOfxImageEffectPropRegionOfInterest: [::core::ffi::c_char; 35] = unsafe {
    ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
        *b"OfxImageEffectPropRegionOfInterest\0",
    )
};
pub const kOfxImageEffectPropRenderWindow: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"OfxImageEffectPropRenderWindow\0",
    )
};
pub const kOfxImageEffectOutputClipName: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"Output\0") };
pub const kOfxImageEffectSimpleSourceClipName: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"Source\0") };
pub const kOfxImageEffectSuite: [::core::ffi::c_char; 20] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_char; 20]>(
        *b"OfxImageEffectSuite\0",
    )
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const kOfxPropertySuite: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"OfxPropertySuite\0")
};
