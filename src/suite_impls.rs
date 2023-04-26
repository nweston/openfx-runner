#![allow(non_snake_case)]
use crate::cstr_to_string;
use crate::suites::*;
use crate::types::*;
use crate::{FromProperty, Handle, OfxError, ParamValue, PropertySet, PropertyValue};
use std::ffi::{c_char, c_double, c_int, c_uint, c_void, CStr};

// ========= ImageEffectSuite =========
extern "C" fn getPropertySet(
    imageEffect: OfxImageEffectHandle,
    propHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    unsafe {
        *propHandle = imageEffect.with_object(|effect| effect.properties.clone().into())
    };
    OfxStatus::OK
}

extern "C" fn getParamSet(
    imageEffect: OfxImageEffectHandle,
    paramSet: *mut OfxParamSetHandle,
) -> OfxStatus {
    unsafe {
        *paramSet = imageEffect.with_object(|effect| effect.param_set.clone().into());
    };
    OfxStatus::OK
}

extern "C" fn clipDefine(
    imageEffect: OfxImageEffectHandle,
    name: *const c_char,
    propertySet: *mut OfxPropertySetHandle,
) -> OfxStatus {
    if let Ok(name_str) = unsafe { CStr::from_ptr(name).to_str() } {
        let props = imageEffect
            .with_object(|effect| effect.create_clip(name_str).lock().properties.clone());
        if !propertySet.is_null() {
            unsafe {
                *propertySet = props.into();
            }
        }
        OfxStatus::OK
    } else {
        OfxStatus::ErrUnknown
    }
}

#[allow(unused_variables)]
extern "C" fn clipGetHandle(
    imageEffect: OfxImageEffectHandle,
    name: *const c_char,
    clip: *mut OfxImageClipHandle,
    propertySet: *mut OfxPropertySetHandle,
) -> OfxStatus {
    if let Ok(name_str) = unsafe { CStr::from_ptr(name).to_str() } {
        imageEffect.with_object(|effect| {
            if let Some(c) = effect.clips.get(name_str) {
                unsafe {
                    *clip = c.clone().into();
                    if !propertySet.is_null() {
                        *propertySet = c.lock().properties.clone().into();
                    }
                }
                OfxStatus::OK
            } else {
                OfxStatus::ErrUnknown
            }
        })
    } else {
        OfxStatus::ErrUnknown
    }
}

#[allow(unused_variables)]
extern "C" fn clipGetPropertySet(
    clip: OfxImageClipHandle,
    propHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    clip.with_object(|c| {
        let handle = c.properties.clone().into();
        unsafe { *propHandle = handle }
    });
    OfxStatus::OK
}

#[allow(unused_variables)]
extern "C" fn clipGetImage(
    clip: OfxImageClipHandle,
    _time: OfxTime,
    _region: *const OfxRectD,
    imageHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    clip.with_object(|c| {
        if let Some(ref image) = c.image {
            unsafe {
                *imageHandle = image.properties.clone().into();
            }
            OfxStatus::OK
        } else {
            OfxStatus::Failed
        }
    })
}

#[allow(unused_variables)]
extern "C" fn clipReleaseImage(_imageHandle: OfxPropertySetHandle) -> OfxStatus {
    OfxStatus::OK
}

#[allow(unused_variables)]
extern "C" fn clipGetRegionOfDefinition(
    clip: OfxImageClipHandle,
    time: OfxTime,
    bounds: *const OfxRectD,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn abort(imageEffect: OfxImageEffectHandle) -> c_int {
    return 0;
}

#[allow(unused_variables)]
extern "C" fn imageMemoryAlloc(
    instanceHandle: OfxImageEffectHandle,
    nBytes: usize,
    memoryHandle: *mut OfxImageMemoryHandle,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn imageMemoryFree(memoryHandle: OfxImageMemoryHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn imageMemoryLock(
    memoryHandle: OfxImageMemoryHandle,
    returnedPtr: *mut *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!")
}
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
    properties
        .with_object(|props| props.set(&cstr_to_string(name), index as usize, value));
    OfxStatus::OK
}

extern "C" fn propSetPointer(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut c_void,
) -> OfxStatus {
    set_property(properties, property, index, value.into())
}

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

extern "C" fn propSetDouble(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: c_double,
) -> OfxStatus {
    set_property(properties, property, index, value.into())
}

extern "C" fn propSetInt(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: c_int,
) -> OfxStatus {
    set_property(properties, property, index, value.into())
}

#[allow(unused_variables)]
extern "C" fn propSetPointerN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!");
}

#[allow(unused_variables)]
extern "C" fn propSetStringN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const *const c_char,
) -> OfxStatus {
    panic!("Not implemented!");
}

#[allow(unused_variables)]
extern "C" fn propSetDoubleN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const c_double,
) -> OfxStatus {
    panic!("Not implemented!");
}

#[allow(unused_variables)]
extern "C" fn propSetIntN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const c_int,
) -> OfxStatus {
    panic!("Not implemented!");
}

fn get_property<T: FromProperty>(
    value: *mut T,
    props: &PropertySet,
    key: &str,
    index: usize,
) -> OfxError {
    let r = props.get(&key, index).and_then(|p| {
        if let Some(v) = FromProperty::from_property(p) {
            unsafe { *value = v };
            Ok(())
        } else {
            match p {
                PropertyValue::Unset => Err(OfxError {
                    message: format!("{} {} not set in {}", key, index, props.name),
                    status: OfxStatus::ErrUnknown,
                }),
                _ => Err(OfxError {
                    message: format!(
                        "{} {} unexpected type: {:?} in {}",
                        key, index, p, props.name
                    ),
                    status: OfxStatus::ErrUnknown,
                }),
            }
        }
    });

    match r {
        Ok(_) => OfxError::ok(),
        Err(e) => e,
    }
}

fn get_property_array<T: FromProperty>(
    value: *mut T,
    props: &PropertySet,
    key: &str,
    count: usize,
) -> OfxError {
    for i in 0..count {
        let result = get_property(unsafe { value.offset(i as isize) }, props, key, i);
        if result.status.failed() {
            return result;
        }
    }
    OfxError::ok()
}

extern "C" fn propGetPointer(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut *const c_void,
) -> OfxStatus {
    properties.with_object(|props| {
        get_property(value, props, &cstr_to_string(property), index as usize)
            .get_status("propGetPointer: ")
    })
}

extern "C" fn propGetString(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut *const c_char,
) -> OfxStatus {
    properties.with_object(|props| {
        get_property(value, props, &cstr_to_string(property), index as usize)
            .get_status("propGetString: ")
    })
}

extern "C" fn propGetDouble(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut c_double,
) -> OfxStatus {
    properties.with_object(|props| {
        get_property(value, props, &cstr_to_string(property), index as usize)
            .get_status("propGetDouble: ")
    })
}

extern "C" fn propGetInt(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut c_int,
) -> OfxStatus {
    properties.with_object(|props| {
        get_property(value, props, &cstr_to_string(property), index as usize)
            .get_status("propGetInt: ")
    })
}

#[allow(unused_variables)]
extern "C" fn propGetPointerN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut *const c_void,
) -> OfxStatus {
    properties.with_object(|props| {
        get_property_array(value, props, &cstr_to_string(property), count as usize)
            .get_status("propGetPointerN: ")
    })
}

#[allow(unused_variables)]
extern "C" fn propGetStringN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut *const c_char,
) -> OfxStatus {
    properties.with_object(|props| {
        get_property_array(value, props, &cstr_to_string(property), count as usize)
            .get_status("propGetStringN: ")
    })
}

#[allow(unused_variables)]
extern "C" fn propGetDoubleN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut c_double,
) -> OfxStatus {
    properties.with_object(|props| {
        get_property_array(value, props, &cstr_to_string(property), count as usize)
            .get_status("propGetDoubleN: ")
    })
}

#[allow(unused_variables)]
extern "C" fn propGetIntN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut c_int,
) -> OfxStatus {
    properties.with_object(|props| {
        get_property_array(value, props, &cstr_to_string(property), count as usize)
            .get_status("propGetIntN: ")
    })
}

#[allow(unused_variables)]
extern "C" fn propReset(
    properties: OfxPropertySetHandle,
    property: *const c_char,
) -> OfxStatus {
    panic!("Not implemented!");
}

extern "C" fn propGetDimension(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: *mut c_int,
) -> OfxStatus {
    let key = cstr_to_string(property);
    properties.with_object(|props| {
        if let Some(values) = props.values.get(&key) {
            unsafe { *count = values.0.len() as i32 }
            OfxStatus::OK
        } else {
            println!("propGetDimension: {} not found in {}", key, props.name);
            OfxStatus::ErrUnknown
        }
    })
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
#[allow(unused_variables)]
extern "C" fn paramDefine(
    paramSet: OfxParamSetHandle,
    paramType: *const c_char,
    name: *const c_char,
    propertySet: *mut OfxPropertySetHandle,
) -> OfxStatus {
    if let (Ok(type_str), Ok(name_str)) = unsafe {
        (
            CStr::from_ptr(paramType).to_str(),
            CStr::from_ptr(name).to_str(),
        )
    } {
        let props = paramSet.with_object(|p| p.create_param(type_str, name_str));
        unsafe { *propertySet = props }
        OfxStatus::OK
    } else {
        OfxStatus::ErrUnknown
    }
}

#[allow(unused_variables)]
extern "C" fn paramGetHandle(
    paramSet: OfxParamSetHandle,
    name: *const c_char,
    param: *mut OfxParamHandle,
    propertySet: *mut OfxPropertySetHandle,
) -> OfxStatus {
    if let Ok(name_str) = unsafe { CStr::from_ptr(name).to_str() } {
        paramSet.with_object(|ps| {
            if let Some(p) = ps.params.get(name_str) {
                unsafe {
                    *param = p.clone().into();
                    if !propertySet.is_null() {
                        *propertySet = p.lock().properties.clone().into();
                    }
                }
                OfxStatus::OK
            } else {
                OfxStatus::ErrUnknown
            }
        })
    } else {
        OfxStatus::ErrUnknown
    }
}

extern "C" fn paramSetGetPropertySet(
    paramSet: OfxParamSetHandle,
    propHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    unsafe { *propHandle = paramSet.with_object(|p| p.properties.clone().into()) };
    OfxStatus::OK
}

#[allow(unused_variables)]
extern "C" fn paramGetPropertySet(
    paramHandle: OfxParamHandle,
    propHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    paramHandle.with_object(|param| unsafe {
        *propHandle = param.properties.clone().into();
    });
    OfxStatus::OK
}

extern "C" {
    fn paramGetValue(paramHandle: OfxParamHandle, ...) -> OfxStatus;
    fn paramGetValueAtTime(paramHandle: OfxParamHandle, time: OfxTime, ...) -> OfxStatus;
    fn paramSetValue(paramHandle: OfxParamHandle, ...) -> OfxStatus;
    fn paramSetValueAtTime(paramHandle: OfxParamHandle, time: OfxTime, ...) -> OfxStatus;
}

#[no_mangle]
pub extern "C" fn param_value_count(paramHandle: OfxParamHandle) -> c_int {
    use ParamValue::*;
    paramHandle.with_object(|p| match p.value {
        Double2D(..) | Integer2D(..) => 2,
        Rgb { .. } | Double3D(..) | Integer3D(..) => 3,
        Rgba { .. } => 4,
        Boolean(_) | Choice(_) | Custom(_) | Double(_) | Integer(_) | String(_) => 1,
        Group | Page | Parametric | PushButton => 0,
    })
}

#[no_mangle]
pub extern "C" fn param_get_value_1(
    paramHandle: OfxParamHandle,
    value: *mut c_void,
) -> OfxStatus {
    use ParamValue::*;
    paramHandle.with_object(|p| match p.value {
        Boolean(b) => unsafe { *(value as *mut c_int) = if b { 1 } else { 0 } },
        Choice(index) => unsafe { *(value as *mut c_int) = index as c_int },
        Custom(ref s) | String(ref s) => unsafe {
            *(value as *mut *const c_char) = s.as_ptr()
        },
        Double(v) => unsafe { *(value as *mut c_double) = v },
        Integer(v) => unsafe { *(value as *mut c_int) = v },
        ref x => panic!("unexpected param value {:?}", x),
    });
    OfxStatus::OK
}

#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn param_get_value_2(
    paramHandle: OfxParamHandle,
    value1: *mut c_void,
    value2: *mut c_void,
) -> OfxStatus {
    use ParamValue::*;
    paramHandle.with_object(|p| match p.value {
        Double2D(x, y) => unsafe {
            *(value1 as *mut c_double) = x;
            *(value2 as *mut c_double) = y;
        },
        Integer2D(x, y) => unsafe {
            *(value1 as *mut c_int) = x;
            *(value2 as *mut c_int) = y;
        },
        ref x => panic!("unexpected param value {:?}", x),
    });
    OfxStatus::OK
}

#[no_mangle]
pub extern "C" fn param_get_value_3(
    paramHandle: OfxParamHandle,
    value1: *mut c_void,
    value2: *mut c_void,
    value3: *mut c_void,
) -> OfxStatus {
    use ParamValue::*;
    paramHandle.with_object(|p| match p.value {
        Double3D(x, y, z) => unsafe {
            *(value1 as *mut c_double) = x;
            *(value2 as *mut c_double) = y;
            *(value3 as *mut c_double) = z;
        },
        Integer3D(x, y, z) => unsafe {
            *(value1 as *mut c_int) = x;
            *(value2 as *mut c_int) = y;
            *(value3 as *mut c_int) = z;
        },
        Rgb(r, g, b) => unsafe {
            *(value1 as *mut c_double) = r;
            *(value2 as *mut c_double) = g;
            *(value3 as *mut c_double) = b;
        },
        ref x => panic!("unexpected param value {:?}", x),
    });
    OfxStatus::OK
}

#[no_mangle]
pub extern "C" fn param_get_value_4(
    paramHandle: OfxParamHandle,
    value1: *mut c_void,
    value2: *mut c_void,
    value3: *mut c_void,
    value4: *mut c_void,
) -> OfxStatus {
    use ParamValue::*;
    paramHandle.with_object(|p| match p.value {
        Rgba(r, g, b, a) => unsafe {
            *(value1 as *mut c_double) = r;
            *(value2 as *mut c_double) = g;
            *(value3 as *mut c_double) = b;
            *(value4 as *mut c_double) = a;
        },
        ref x => panic!("unexpected param value {:?}", x),
    });
    OfxStatus::OK
}

#[no_mangle]
pub extern "C" fn param_get_type(handle: OfxParamHandle) -> *const c_char {
    handle.with_object(|p| {
        if let Ok(PropertyValue::String(s)) = p
            .properties
            .lock()
            .get(crate::constants::param::OfxParamPropType, 0)
        {
            s.as_c_str().as_ptr()
        } else {
            panic!("OfxParamPropType not found on param")
        }
    })
}

#[no_mangle]
pub extern "C" fn param_set_value_boolean(handle: OfxParamHandle, value: i32) {
    handle.with_object(|p| p.value = ParamValue::Boolean(value != 0));
}

#[no_mangle]
pub extern "C" fn param_set_value_integer(handle: OfxParamHandle, value: i32) {
    handle.with_object(|p| p.value = ParamValue::Integer(value));
}

#[no_mangle]
pub extern "C" fn param_set_value_choice(handle: OfxParamHandle, value: i32) {
    handle.with_object(|p| p.value = ParamValue::Choice(value as usize));
}

#[no_mangle]
pub extern "C" fn param_set_value_double(handle: OfxParamHandle, value: f64) {
    handle.with_object(|p| p.value = ParamValue::Double(value));
}

#[no_mangle]
pub extern "C" fn param_set_value_string(handle: OfxParamHandle, value: *const c_char) {
    handle.with_object(|p| {
        p.value = ParamValue::String(unsafe { CStr::from_ptr(value) }.into())
    });
}

#[allow(unused_variables)]
extern "C" fn paramGetDerivative(
    paramHandle: OfxParamHandle,
    time: OfxTime,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn paramGetIntegral(
    paramHandle: OfxParamHandle,
    time1: OfxTime,
    time2: OfxTime,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn paramGetNumKeys(
    paramHandle: OfxParamHandle,
    numberOfKeys: *mut c_uint,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn paramGetKeyTime(
    paramHandle: OfxParamHandle,
    nthKey: c_uint,
    time: *mut OfxTime,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn paramGetKeyIndex(
    paramHandle: OfxParamHandle,
    time: OfxTime,
    direction: c_int,
    index: *mut c_int,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn paramDeleteKey(paramHandle: OfxParamHandle, time: OfxTime) -> OfxStatus {
    OfxStatus::OK
}

#[allow(unused_variables)]
extern "C" fn paramDeleteAllKeys(paramHandle: OfxParamHandle) -> OfxStatus {
    OfxStatus::OK
}

#[allow(unused_variables)]
extern "C" fn paramCopy(
    paramTo: OfxParamHandle,
    paramFrom: OfxParamHandle,
    dstOffset: OfxTime,
    frameRange: *const OfxRangeD,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn paramEditBegin(
    paramSet: OfxParamSetHandle,
    name: *const c_char,
) -> OfxStatus {
    OfxStatus::OK
}

#[allow(unused_variables)]
extern "C" fn paramEditEnd(paramSet: OfxParamSetHandle) -> OfxStatus {
    OfxStatus::OK
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
extern "C" fn message(
    _handle: *mut c_void,
    messageType: *const c_char,
    messageId: *const c_char,
    format: *const c_char,
) -> OfxStatus {
    unsafe {
        println!(
            "\n{}: {}. {}\n",
            CStr::from_ptr(messageType).to_str().unwrap(),
            if messageId.is_null() {
                "(null)"
            } else {
                CStr::from_ptr(messageId).to_str().unwrap()
            },
            CStr::from_ptr(format).to_str().unwrap()
        );
    }
    OfxStatus::OK
}

pub const MESSAGE_SUITE: OfxMessageSuiteV1 = OfxMessageSuiteV1 { message };

// ========= Memory suite =========
#[allow(unused_variables)]
extern "C" fn memoryAlloc(
    handle: *mut c_void,
    nBytes: usize,
    allocatedData: *mut *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn memoryFree(allocatedData: *mut c_void) -> OfxStatus {
    panic!("Not implemented!")
}

pub const MEMORY_SUITE: OfxMemorySuiteV1 = OfxMemorySuiteV1 {
    memoryAlloc,
    memoryFree,
};

// ========= Multithread suite =========

#[allow(unused_variables)]
extern "C" fn multiThread(
    func: OfxThreadFunctionV1,
    nThreads: c_uint,
    customArg: *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn multiThreadNumCPUs(nCPUs: *mut c_int) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn multiThreadIndex(threadIndex: *mut c_int) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn multiThreadIsSpawnedThread() -> c_int {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn mutexCreate(mutex: OfxMutexHandle, lockCount: c_int) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn mutexDestroy(mutex: OfxMutexConstHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn mutexLock(mutex: OfxMutexConstHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn mutexUnLock(mutex: OfxMutexConstHandle) -> OfxStatus {
    panic!("Not implemented!")
}

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
