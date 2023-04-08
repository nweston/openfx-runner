use crate::cstr_to_string;
use crate::suites::*;
use crate::types::*;
use crate::{Handle, PropertyValue};
use std::ffi::{c_char, c_double, c_int, c_uint, c_void, CStr};

// ========= ImageEffectSuite =========
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn getPropertySet(
    imageEffect: OfxImageEffectHandle,
    propHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    unsafe { *propHandle = (&mut imageEffect.as_ref().properties).into() };
    OfxStatus::OK
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn getParamSet(
    imageEffect: OfxImageEffectHandle,
    paramSet: *mut OfxParamSetHandle,
) -> OfxStatus {
    unsafe {
        *paramSet = (&mut imageEffect.as_ref().params).into();
    };
    OfxStatus::OK
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

pub const IMAGE_EFFECT_SUITE: OfxImageEffectSuiteV1 = OfxImageEffectSuiteV1 {
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
fn set_property(
    properties: OfxPropertySetHandle,
    name: *const c_char,
    index: c_int,
    value: PropertyValue,
) -> OfxStatus {
    if let Ok(name_str) = unsafe { CStr::from_ptr(name).to_str() } {
        let prop = properties
            .as_ref()
            .0
            .entry(name_str.to_string())
            .or_insert(Default::default());
        let uindex = index as usize;
        if uindex >= prop.0.len() {
            prop.0.resize_with(uindex + 1, || PropertyValue::Unset)
        }
        prop.0[uindex] = value;
        OfxStatus::OK
    } else {
        OfxStatus::ErrUnknown
    }
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propSetPointer(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut c_void,
) -> OfxStatus {
    set_property(properties, property, index, value.into())
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propSetString(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *const c_char,
) -> OfxStatus {
    if let Ok(value_str) = unsafe { CStr::from_ptr(value).to_str() } {
        set_property(properties, property, index, value_str.into())
    } else {
        OfxStatus::ErrUnknown
    }
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propSetDouble(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: c_double,
) -> OfxStatus {
    set_property(properties, property, index, value.into())
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propSetInt(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: c_int,
) -> OfxStatus {
    set_property(properties, property, index, value.into())
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
    let key = unsafe { cstr_to_string(property) };
    let props = properties.as_ref();
    if let Some(values) = props.0.get(&key) {
        if let Some(v) = values.0.get(index as usize) {
            match v {
                PropertyValue::Pointer(p) => unsafe {
                    *value = *p;
                    OfxStatus::OK
                },
                PropertyValue::Unset => {
                    println!("propGetPointer: {} {} not set", key, index);
                    OfxStatus::ErrUnknown
                }
                _ => {
                    println!(
                        "propGetPointer: {} {} unexpected type: {:?}",
                        key, index, v
                    );
                    OfxStatus::ErrUnknown
                }
            }
        } else {
            println!("propGetPointer: {} {}: bad index", key, index);
            OfxStatus::ErrUnknown
        }
    } else {
        println!("propGetPointer: {} not found", key);
        OfxStatus::ErrUnknown
    }
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
extern "C" fn propGetString(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut *const c_char,
) -> OfxStatus {
    let key = unsafe { cstr_to_string(property) };
    let props = properties.as_ref();
    if let Some(values) = props.0.get(&key) {
        if let Some(v) = values.0.get(index as usize) {
            match v {
                PropertyValue::String(s) => unsafe {
                    *value = s.as_ptr();
                    OfxStatus::OK
                },
                PropertyValue::Unset => {
                    println!("propGetString: {} {} not set", key, index);
                    OfxStatus::ErrUnknown
                }
                _ => {
                    println!("propGetString: {} {} unexpected type: {:?}", key, index, v);
                    OfxStatus::ErrUnknown
                }
            }
        } else {
            println!("propGetString: {} {}: bad index", key, index);
            OfxStatus::ErrUnknown
        }
    } else {
        println!("propGetString: {} not found", key);
        OfxStatus::ErrUnknown
    }
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
    let key = unsafe { cstr_to_string(property) };
    let props = properties.as_ref();
    if let Some(values) = props.0.get(&key) {
        if let Some(v) = values.0.get(index as usize) {
            match v {
                PropertyValue::Int(i) => unsafe {
                    *value = *i;
                    OfxStatus::OK
                },
                PropertyValue::Unset => {
                    println!("propGetInt: {} {} not set", key, index);
                    OfxStatus::ErrUnknown
                }
                _ => {
                    println!("propGetInt: {} {} unexpected type: {:?}", key, index, v);
                    OfxStatus::ErrUnknown
                }
            }
        } else {
            println!("propGetInt: {} {}: bad index", key, index);
            OfxStatus::ErrUnknown
        }
    } else {
        println!("propGetInt: {} not found", key);
        OfxStatus::ErrUnknown
    }
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
    value: *mut *const c_char,
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
    let key = unsafe { cstr_to_string(property) };
    let props = properties.as_ref();
    if let Some(values) = props.0.get(&key) {
        unsafe { *count = values.0.len() as i32 }
        OfxStatus::OK
    } else {
        println!("propGetDimension: {} not found", key);
        OfxStatus::ErrUnknown
    }
}

pub const PROPERTY_SUITE: OfxPropertySuiteV1 = OfxPropertySuiteV1 {
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
    unsafe { *propHandle = (&mut paramSet.as_ref().properties).into() };
    OfxStatus::OK
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

pub const PARAMETER_SUITE: OfxParameterSuiteV1 = OfxParameterSuiteV1 {
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

pub const MESSAGE_SUITE: OfxMessageSuiteV1 = OfxMessageSuiteV1 { message };

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

pub const MEMORY_SUITE: OfxMemorySuiteV1 = OfxMemorySuiteV1 {
    memoryAlloc,
    memoryFree,
};

// ========= Multithread suite =========

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

pub const MULTI_THREAD_SUITE: OfxMultiThreadSuiteV1 = OfxMultiThreadSuiteV1 {
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
