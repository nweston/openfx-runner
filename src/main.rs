use std::error::Error;
use std::ffi::{c_char, c_double, c_int, c_uint, c_void, CStr, CString};
use std::fs;

struct OfxPropertySet {}
type OfxPropertySetHandle = *mut OfxPropertySet;
type OfxImageEffectHandle = *mut c_void;
type OfxParamSetHandle = *mut c_void;
type OfxParamHandle = *mut c_void;
type OfxImageClipHandle = *mut c_void;
type OfxImageMemoryHandle = *mut c_void;
type OfxMutexHandle = *mut c_void;
type OfxMutexConstHandle = *const c_void;
type OfxStatus = c_int;
type OfxTime = c_double;
#[allow(dead_code)]
#[repr(C)]
struct OfxRectD {
    x1: c_double,
    y1: c_double,
    x2: c_double,
    y2: c_double,
}
#[allow(dead_code)]
#[repr(C)]
struct OfxRangeD {
    min: c_double,
    max: c_double,
}

#[allow(non_snake_case)]
#[repr(C)]
struct OfxHost {
    host: OfxPropertySetHandle,
    fetchSuite:
        extern "C" fn(OfxPropertySetHandle, *const c_char, c_int) -> *const c_void,
}

#[allow(non_snake_case)]
#[repr(C)]
struct OfxPluginRaw {
    pluginApi: *const c_char,
    apiVersion: c_int,
    pluginIdentifier: *const c_char,
    pluginVersionMajor: c_uint,
    pluginVersionMinor: c_uint,
    setHost: extern "C" fn(*const OfxHost),
    mainEntry: extern "C" fn(
        *const c_char,
        *const c_void,
        OfxPropertySetHandle,
        OfxPropertySetHandle,
    ) -> OfxStatus,
}

#[derive(Debug)]
#[allow(dead_code)]
struct OfxPlugin {
    plugin_api: String,
    api_version: i32,
    plugin_identifier: String,
    plugin_version_major: u32,
    plugin_version_minor: u32,
    set_host: extern "C" fn(*const OfxHost),
    main_entry: extern "C" fn(
        *const c_char,
        *const c_void,
        OfxPropertySetHandle,
        OfxPropertySetHandle,
    ) -> OfxStatus,
}

impl OfxPlugin {
    fn call_action(
        &self,
        action: &str,
        handle: *const c_void,
        in_args: OfxPropertySetHandle,
        out_args: OfxPropertySetHandle,
    ) -> OfxStatus {
        let c_action = CString::new(action).unwrap();
        (self.main_entry)(c_action.as_ptr(), handle, in_args, out_args)
    }
}

// ========= ImageEffectSuite =========
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn getPropertySet(
    imageEffect: OfxImageEffectHandle,
    propHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn getParamSet(
    imageEffect: OfxImageEffectHandle,
    paramSet: *mut OfxParamSetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn clipDefine(
    imageEffect: OfxImageEffectHandle,
    name: *const char,
    propertySet: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn clipGetHandle(
    imageEffect: OfxImageEffectHandle,
    name: *const char,
    clip: *mut OfxImageClipHandle,
    propertySet: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn clipGetPropertySet(
    clip: OfxImageClipHandle,
    propHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn clipGetImage(
    clip: OfxImageClipHandle,
    time: OfxTime,
    region: *const OfxRectD,
    imageHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn clipReleaseImage(imageHandle: OfxPropertySetHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn clipGetRegionOfDefinition(
    clip: OfxImageClipHandle,
    time: OfxTime,
    bounds: *const OfxRectD,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn abort(imageEffect: OfxImageEffectHandle) -> c_int {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn imageMemoryAlloc(
    instanceHandle: OfxImageEffectHandle,
    nBytes: usize,
    memoryHandle: *mut OfxImageMemoryHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn imageMemoryFree(memoryHandle: OfxImageMemoryHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn imageMemoryLock(
    memoryHandle: OfxImageMemoryHandle,
    returnedPtr: *mut *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn imageMemoryUnlock(memoryHandle: OfxImageMemoryHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[repr(C)]
struct OfxImageEffectSuiteV1 {
    getPropertySet: extern "C" fn(
        imageEffect: OfxImageEffectHandle,
        propHandle: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    getParamSet: extern "C" fn(
        imageEffect: OfxImageEffectHandle,
        paramSet: *mut OfxParamSetHandle,
    ) -> OfxStatus,
    clipDefine: extern "C" fn(
        imageEffect: OfxImageEffectHandle,
        name: *const char,
        propertySet: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    clipGetHandle: extern "C" fn(
        imageEffect: OfxImageEffectHandle,
        name: *const char,
        clip: *mut OfxImageClipHandle,
        propertySet: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    clipGetPropertySet: extern "C" fn(
        clip: OfxImageClipHandle,
        propHandle: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    clipGetImage: extern "C" fn(
        clip: OfxImageClipHandle,
        time: OfxTime,
        region: *const OfxRectD,
        imageHandle: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    clipReleaseImage: extern "C" fn(imageHandle: OfxPropertySetHandle) -> OfxStatus,
    clipGetRegionOfDefinition: extern "C" fn(
        clip: OfxImageClipHandle,
        time: OfxTime,
        bounds: *const OfxRectD,
    ) -> OfxStatus,
    abort: extern "C" fn(imageEffect: OfxImageEffectHandle) -> c_int,
    imageMemoryAlloc: extern "C" fn(
        instanceHandle: OfxImageEffectHandle,
        nBytes: usize,
        memoryHandle: *mut OfxImageMemoryHandle,
    ) -> OfxStatus,
    imageMemoryFree: extern "C" fn(memoryHandle: OfxImageMemoryHandle) -> OfxStatus,
    imageMemoryLock: extern "C" fn(
        memoryHandle: OfxImageMemoryHandle,
        returnedPtr: *mut *mut c_void,
    ) -> OfxStatus,
    imageMemoryUnlock: extern "C" fn(memoryHandle: OfxImageMemoryHandle) -> OfxStatus,
}

const IMAGE_EFFECT_SUITE: OfxImageEffectSuiteV1 = OfxImageEffectSuiteV1 {
    getPropertySet,
    getParamSet,
    clipDefine,
    clipGetHandle,
    clipGetPropertySet,
    clipGetImage,
    clipReleaseImage,
    clipGetRegionOfDefinition,
    abort,
    imageMemoryAlloc,
    imageMemoryFree,
    imageMemoryLock,
    imageMemoryUnlock,
};

// ========= Property Suite =========
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propSetPointer(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propSetString(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *const c_char,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propSetDouble(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: c_double,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propSetInt(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: c_int,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propSetPointerN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propSetStringN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const *const c_char,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propSetDoubleN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const c_double,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propSetIntN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const c_int,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propGetPointer(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propGetString(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut *mut c_char,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propGetDouble(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut c_double,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propGetInt(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut c_int,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propGetPointerN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propGetStringN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut *mut c_char,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propGetDoubleN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut c_double,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propGetIntN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut c_int,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propReset(
    properties: OfxPropertySetHandle,
    property: *const c_char,
) -> OfxStatus {
    panic!("Not implemented!");
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propGetDimension(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: *mut c_int,
) -> OfxStatus {
    panic!("Not implemented!");
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[repr(C)]
struct OfxPropertySuiteV1 {
    propSetPointer: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut c_void,
    ) -> OfxStatus,
    propSetString: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *const c_char,
    ) -> OfxStatus,
    propSetDouble: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: c_double,
    ) -> OfxStatus,
    propSetInt: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: c_int,
    ) -> OfxStatus,
    propSetPointerN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *const *mut c_void,
    ) -> OfxStatus,
    propSetStringN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *const *const c_char,
    ) -> OfxStatus,
    propSetDoubleN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *const c_double,
    ) -> OfxStatus,
    propSetIntN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *const c_int,
    ) -> OfxStatus,
    propGetPointer: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut *mut c_void,
    ) -> OfxStatus,
    propGetString: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut *mut c_char,
    ) -> OfxStatus,
    propGetDouble: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut c_double,
    ) -> OfxStatus,
    propGetInt: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut c_int,
    ) -> OfxStatus,
    propGetPointerN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *mut *mut c_void,
    ) -> OfxStatus,
    propGetStringN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *mut *mut c_char,
    ) -> OfxStatus,
    propGetDoubleN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *mut c_double,
    ) -> OfxStatus,
    propGetIntN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *mut c_int,
    ) -> OfxStatus,
    propReset: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
    ) -> OfxStatus,
    propGetDimension: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: *mut c_int,
    ) -> OfxStatus,
}

const PROPERTY_SUITE: OfxPropertySuiteV1 = OfxPropertySuiteV1 {
    propSetPointer,
    propSetString,
    propSetDouble,
    propSetInt,
    propSetPointerN,
    propSetStringN,
    propSetDoubleN,
    propSetIntN,
    propGetPointer,
    propGetString,
    propGetDouble,
    propGetInt,
    propGetPointerN,
    propGetStringN,
    propGetDoubleN,
    propGetIntN,
    propReset,
    propGetDimension,
};

// ========= Parameter suite =========
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramDefine(
    paramSet: OfxParamSetHandle,
    paramType: *const c_char,
    name: *const c_char,
    propertySet: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramGetHandle(
    paramSet: OfxParamSetHandle,
    name: *const c_char,
    param: *mut OfxParamHandle,
    propertySet: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramSetGetPropertySet(
    paramSet: OfxParamSetHandle,
    propHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramGetPropertySet(
    paramHandle: OfxParamHandle,
    propHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramGetValue(paramHandle: OfxParamHandle) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramGetValueAtTime(
    paramHandle: OfxParamHandle,
    time: OfxTime,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramGetDerivative(
    paramHandle: OfxParamHandle,
    time: OfxTime,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramGetIntegral(
    paramHandle: OfxParamHandle,
    time1: OfxTime,
    time2: OfxTime,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramSetValue(paramHandle: OfxParamHandle) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramSetValueAtTime(
    paramHandle: OfxParamHandle,
    time: OfxTime, // time in frames
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramGetNumKeys(
    paramHandle: OfxParamHandle,
    numberOfKeys: *mut c_uint,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramGetKeyTime(
    paramHandle: OfxParamHandle,
    nthKey: c_uint,
    time: *mut OfxTime,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramGetKeyIndex(
    paramHandle: OfxParamHandle,
    time: OfxTime,
    direction: c_int,
    index: *mut c_int,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramDeleteKey(paramHandle: OfxParamHandle, time: OfxTime) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramDeleteAllKeys(paramHandle: OfxParamHandle) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramCopy(
    paramTo: OfxParamHandle,
    paramFrom: OfxParamHandle,
    dstOffset: OfxTime,
    frameRange: *const OfxRangeD,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramEditBegin(
    paramSet: OfxParamSetHandle,
    name: *const c_char,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn paramEditEnd(paramSet: OfxParamSetHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[repr(C)]
struct OfxParameterSuiteV1 {
    paramDefine: extern "C" fn(
        paramSet: OfxParamSetHandle,
        paramType: *const c_char,
        name: *const c_char,
        propertySet: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    paramGetHandle: extern "C" fn(
        paramSet: OfxParamSetHandle,
        name: *const c_char,
        param: *mut OfxParamHandle,
        propertySet: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    paramSetGetPropertySet: extern "C" fn(
        paramSet: OfxParamSetHandle,
        propHandle: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    paramGetPropertySet: extern "C" fn(
        paramHandle: OfxParamHandle,
        propHandle: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    // XXX: all GetValue functions use varargs to return values
    paramGetValue: extern "C" fn(paramHandle: OfxParamHandle) -> OfxStatus,
    paramGetValueAtTime:
        extern "C" fn(paramHandle: OfxParamHandle, time: OfxTime) -> OfxStatus,
    paramGetDerivative:
        extern "C" fn(paramHandle: OfxParamHandle, time: OfxTime) -> OfxStatus,
    paramGetIntegral: extern "C" fn(
        paramHandle: OfxParamHandle,
        time1: OfxTime,
        time2: OfxTime,
    ) -> OfxStatus,
    paramSetValue: extern "C" fn(paramHandle: OfxParamHandle) -> OfxStatus,
    paramSetValueAtTime: extern "C" fn(
        paramHandle: OfxParamHandle,
        time: OfxTime, // time in frames
    ) -> OfxStatus,
    paramGetNumKeys: extern "C" fn(
        paramHandle: OfxParamHandle,
        numberOfKeys: *mut c_uint,
    ) -> OfxStatus,
    paramGetKeyTime: extern "C" fn(
        paramHandle: OfxParamHandle,
        nthKey: c_uint,
        time: *mut OfxTime,
    ) -> OfxStatus,
    paramGetKeyIndex: extern "C" fn(
        paramHandle: OfxParamHandle,
        time: OfxTime,
        direction: c_int,
        index: *mut c_int,
    ) -> OfxStatus,
    paramDeleteKey:
        extern "C" fn(paramHandle: OfxParamHandle, time: OfxTime) -> OfxStatus,
    paramDeleteAllKeys: extern "C" fn(paramHandle: OfxParamHandle) -> OfxStatus,
    paramCopy: extern "C" fn(
        paramTo: OfxParamHandle,
        paramFrom: OfxParamHandle,
        dstOffset: OfxTime,
        frameRange: *const OfxRangeD,
    ) -> OfxStatus,
    paramEditBegin:
        extern "C" fn(paramSet: OfxParamSetHandle, name: *const c_char) -> OfxStatus,
    paramEditEnd: extern "C" fn(paramSet: OfxParamSetHandle) -> OfxStatus,
}

const PARAMETER_SUITE: OfxParameterSuiteV1 = OfxParameterSuiteV1 {
    paramDefine,
    paramGetHandle,
    paramSetGetPropertySet,
    paramGetPropertySet,
    paramGetValue,
    paramGetValueAtTime,
    paramGetDerivative,
    paramGetIntegral,
    paramSetValue,
    paramSetValueAtTime,
    paramGetNumKeys,
    paramGetKeyTime,
    paramGetKeyIndex,
    paramDeleteKey,
    paramDeleteAllKeys,
    paramCopy,
    paramEditBegin,
    paramEditEnd,
};

// ========= MessageSuiteV1 =========
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn message(
    handle: *mut c_void,
    messageType: *const c_char,
    messageId: *const c_char,
    format: *const c_char,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[repr(C)]
struct OfxMessageSuiteV1 {
    // XXX: uses varargs
    message: extern "C" fn(
        handle: *mut c_void,
        messageType: *const c_char,
        messageId: *const c_char,
        format: *const c_char,
    ) -> OfxStatus,
}

const MESSAGE_SUITE: OfxMessageSuiteV1 = OfxMessageSuiteV1 { message };

// ========= Memory suite =========
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn memoryAlloc(
    handle: *mut c_void,
    nBytes: usize,
    allocatedData: *mut *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn memoryFree(allocatedData: *mut c_void) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[repr(C)]
struct OfxMemorySuiteV1 {
    memoryAlloc: extern "C" fn(
        handle: *mut c_void,
        nBytes: usize,
        allocatedData: *mut *mut c_void,
    ) -> OfxStatus,
    memoryFree: extern "C" fn(allocatedData: *mut c_void) -> OfxStatus,
}

const MEMORY_SUITE: OfxMemorySuiteV1 = OfxMemorySuiteV1 {
    memoryAlloc,
    memoryFree,
};

// ========= Multithread suite =========

type OfxThreadFunctionV1 = extern "C" fn(
    threadIndex: c_uint,
    threadMax: c_uint,
    customArg: *mut c_void,
) -> OfxStatus;

#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn multiThread(
    func: OfxThreadFunctionV1,
    nThreads: c_uint,
    customArg: *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn multiThreadNumCPUs(nCPUs: *mut c_int) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn multiThreadIndex(threadIndex: *mut c_int) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn multiThreadIsSpawnedThread() -> c_int {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn mutexCreate(mutex: OfxMutexHandle, lockCount: c_int) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn mutexDestroy(mutex: OfxMutexConstHandle) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn mutexLock(mutex: OfxMutexConstHandle) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn mutexUnLock(mutex: OfxMutexConstHandle) -> OfxStatus {
    panic!("Not implemented!")
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn mutexTryLock(mutex: OfxMutexConstHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(non_snake_case)]
#[allow(dead_code)]
#[repr(C)]
struct OfxMultiThreadSuiteV1 {
    multiThread: extern "C" fn(
        func: OfxThreadFunctionV1,
        nThreads: c_uint,
        customArg: *mut c_void,
    ) -> OfxStatus,
    multiThreadNumCPUs: extern "C" fn(nCPUs: *mut c_int) -> OfxStatus,
    multiThreadIndex: extern "C" fn(threadIndex: *mut c_int) -> OfxStatus,
    multiThreadIsSpawnedThread: extern "C" fn() -> c_int,
    mutexCreate: extern "C" fn(mutex: OfxMutexHandle, lockCount: c_int) -> OfxStatus,
    mutexDestroy: extern "C" fn(mutex: OfxMutexConstHandle) -> OfxStatus,
    mutexLock: extern "C" fn(mutex: OfxMutexConstHandle) -> OfxStatus,
    mutexUnLock: extern "C" fn(mutex: OfxMutexConstHandle) -> OfxStatus,
    mutexTryLock: extern "C" fn(mutex: OfxMutexConstHandle) -> OfxStatus,
}

const MULTI_THREAD_SUITE: OfxMultiThreadSuiteV1 = OfxMultiThreadSuiteV1 {
    multiThread,
    multiThreadNumCPUs,
    multiThreadIndex,
    multiThreadIsSpawnedThread,
    mutexCreate,
    mutexDestroy,
    mutexLock,
    mutexUnLock,
    mutexTryLock,
};

// ========= End of suites =========
fn plist_path(bundle_path: &std::path::Path) -> std::path::PathBuf {
    return bundle_path.join("Contents/Info.plist");
}

struct OfxBundle {
    path: std::path::PathBuf,
    plist: plist::Value,
}

fn make_bundle(path: std::path::PathBuf) -> Result<OfxBundle, Box<dyn Error>> {
    let plist = plist::Value::from_file(plist_path(&path))?;
    return Ok(OfxBundle { path, plist });
}

fn library_path(bundle: &OfxBundle) -> std::path::PathBuf {
    let lib = bundle
        .plist
        .as_dictionary()
        .unwrap()
        .get("CFBundleExecutable")
        .unwrap()
        .as_string()
        .unwrap();
    if cfg!(target_os = "linux") {
        return bundle.path.join("Contents/Linux-x86-64").join(lib);
    } else if cfg!(windows) {
        return bundle.path.join("Contents/Win64").join(lib);
    } else {
        return bundle.path.join("Contents/MacOS").join(lib);
    }
}

fn ofx_bundles() -> Vec<OfxBundle> {
    if let Ok(dir) = fs::read_dir("/usr/OFX/Plugins/") {
        let x = dir.filter_map(|entry| {
            let path: std::path::PathBuf = entry.ok()?.path();
            if path.is_dir() {
                if let Some(f) = path.file_name() {
                    if f.to_str().map_or(false, |s| s.ends_with(".ofx.bundle")) {
                        return Some(make_bundle(path).ok()?);
                    }
                }
            }
            return None;
        });
        return x.collect();
    }
    return Vec::new();
}

unsafe fn cstr_to_string(s: *const c_char) -> String {
    CStr::from_ptr(s).to_str().unwrap().to_string()
}

#[allow(unused_variables)]
extern "C" fn fetch_suite(
    host: OfxPropertySetHandle,
    name: *const c_char,
    version: c_int,
) -> *const c_void {
    let suite = unsafe { CStr::from_ptr(name).to_str().unwrap() };
    if suite == "OfxImageEffectSuite" {
        assert!(version == 1);
        &IMAGE_EFFECT_SUITE as *const _ as *const c_void
    } else if suite == "OfxPropertySuite" {
        assert!(version == 1);
        &PROPERTY_SUITE as *const _ as *const c_void
    } else if suite == "OfxParameterSuite" {
        assert!(version == 1);
        &PARAMETER_SUITE as *const _ as *const c_void
    } else if suite == "OfxMemorySuite" {
        assert!(version == 1);
        &MEMORY_SUITE as *const _ as *const c_void
    } else if suite == "OfxMultiThreadSuite" {
        assert!(version == 1);
        &MULTI_THREAD_SUITE as *const _ as *const c_void
    } else if suite == "OfxMessageSuite" {
        assert!(version == 1);
        &MESSAGE_SUITE as *const _ as *const c_void
    } else {
        println!("fetch_suite: {} v{} is not available", suite, version);
        std::ptr::null()
    }
}

fn main() {
    let mut host_props = OfxPropertySet {};
    let host = OfxHost {
        host: &mut host_props,
        fetchSuite: fetch_suite,
    };

    for bundle in ofx_bundles() {
        let count;
        let mut plugins = Vec::new();

        unsafe {
            let lib = libloading::Library::new(library_path(&bundle)).unwrap();
            let func: libloading::Symbol<unsafe extern "C" fn() -> i32> =
                lib.get(b"OfxGetNumberOfPlugins").unwrap();
            count = func();
            let func2: libloading::Symbol<
                unsafe extern "C" fn(i32) -> *const OfxPluginRaw,
            > = lib.get(b"OfxGetPlugin").unwrap();
            for i in 0..count {
                let p = &*func2(i);
                plugins.push(OfxPlugin {
                    plugin_api: cstr_to_string(p.pluginApi),
                    api_version: p.apiVersion,
                    plugin_identifier: cstr_to_string(p.pluginIdentifier),
                    plugin_version_major: p.pluginVersionMajor,
                    plugin_version_minor: p.pluginVersionMinor,
                    set_host: p.setHost,
                    main_entry: p.mainEntry,
                })
            }
        }
        println!(
            "{}, {} => {}",
            bundle.path.display(),
            library_path(&bundle).display(),
            count
        );
        for p in plugins {
            (p.set_host)(&host);
            let stat = p.call_action(
                "OfxActionLoad",
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            let stat2 = p.call_action(
                "OfxActionUnload",
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            println!(
                "  {:?}, Load returned {}, Unload returned {}",
                p, stat, stat2
            );
        }
        println!()
    }
}
