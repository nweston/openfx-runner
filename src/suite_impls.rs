#![allow(non_snake_case)]
use crate::strings::OfxStr;
use crate::suites::*;
use crate::types::*;
use crate::{FromProperty, Handle, OfxError, ParamValue, PropertySet, PropertyValue};
use libc::{free, posix_memalign};
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
    let props = imageEffect.with_object(|effect| {
        effect
            .create_clip(OfxStr::from_ptr(name))
            .lock()
            .properties
            .clone()
    });
    if !propertySet.is_null() {
        unsafe {
            *propertySet = props.into();
        }
    }
    OfxStatus::OK
}

#[allow(unused_variables)]
extern "C" fn clipGetHandle(
    imageEffect: OfxImageEffectHandle,
    name: *const c_char,
    clip: *mut OfxImageClipHandle,
    propertySet: *mut OfxPropertySetHandle,
) -> OfxStatus {
    imageEffect.with_object(|effect| {
        if let Some(c) = effect.clips.get(OfxStr::from_ptr(name).as_str()) {
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
    time: OfxTime,
    _region: *const OfxRectD,
    imageHandle: *mut OfxPropertySetHandle,
) -> OfxStatus {
    clip.with_object(|c| {
        if let Some(ref image) = c.images.image_at_time(time) {
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
    bounds: *mut OfxRectD,
) -> OfxStatus {
    clip.with_object(|c| {
        if let Some(rod) = c.region_of_definition {
            unsafe {
                *bounds = rod;
            }
            OfxStatus::OK
        } else {
            OfxStatus::Failed
        }
    })
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
    unsafe {
        // Allocate memory directly and return the pointer as a
        // handle.
        let mut ptr: *mut c_void = std::ptr::null_mut();
        // 16-byte alignment is required by the spec
        if posix_memalign(&mut ptr, 16, nBytes) != 0 {
            return OfxStatus::ErrMemory;
        }
        *memoryHandle = ptr.into();
    };
    OfxStatus::OK
}

#[allow(unused_variables)]
extern "C" fn imageMemoryFree(memoryHandle: OfxImageMemoryHandle) -> OfxStatus {
    unsafe {
        free(memoryHandle.into());
    };
    OfxStatus::OK
}

#[allow(unused_variables)]
extern "C" fn imageMemoryLock(
    memoryHandle: OfxImageMemoryHandle,
    returnedPtr: *mut *mut c_void,
) -> OfxStatus {
    // The handle is already a pointer to allocated memory, just
    // return it
    unsafe {
        *returnedPtr = memoryHandle.into();
    }
    OfxStatus::OK
}
#[allow(unused_variables)]
extern "C" fn imageMemoryUnlock(memoryHandle: OfxImageMemoryHandle) -> OfxStatus {
    // Nothing to do
    OfxStatus::OK
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
    properties.with_object(|props| {
        props.set(OfxStr::from_ptr(name).as_str(), index as usize, value)
    });
    OfxStatus::OK
}

fn set_property_n<T: Into<PropertyValue> + Copy>(
    properties: OfxPropertySetHandle,
    name: *const c_char,
    count: c_int,
    value: *const T,
) -> OfxStatus {
    let s = unsafe { std::slice::from_raw_parts(value, count as usize) };
    for (i, v) in s.iter().enumerate() {
        set_property(properties, name, i as i32, (*v).into());
    }
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
    set_property(properties, property, index, value.into())
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
    set_property_n(properties, property, count, value)
}

#[allow(unused_variables)]
extern "C" fn propSetStringN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const *const c_char,
) -> OfxStatus {
    set_property_n(properties, property, count, value)
}

#[allow(unused_variables)]
extern "C" fn propSetDoubleN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const c_double,
) -> OfxStatus {
    set_property_n(properties, property, count, value)
}

#[allow(unused_variables)]
extern "C" fn propSetIntN(
    properties: OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const c_int,
) -> OfxStatus {
    set_property_n(properties, property, count, value)
}

fn get_property<T: FromProperty>(
    value: *mut T,
    props: &PropertySet,
    key: OfxStr,
    index: usize,
) -> OfxError {
    let r = props.get(key, index).and_then(|p| {
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
    key: OfxStr,
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
        get_property(value, props, OfxStr::from_ptr(property), index as usize)
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
        get_property(value, props, OfxStr::from_ptr(property), index as usize)
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
        get_property(value, props, OfxStr::from_ptr(property), index as usize)
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
        get_property(value, props, OfxStr::from_ptr(property), index as usize)
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
        get_property_array(value, props, OfxStr::from_ptr(property), count as usize)
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
        get_property_array(value, props, OfxStr::from_ptr(property), count as usize)
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
        get_property_array(value, props, OfxStr::from_ptr(property), count as usize)
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
        get_property_array(value, props, OfxStr::from_ptr(property), count as usize)
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
    let key = OfxStr::from_ptr(property);
    properties.with_object(|props| {
        if let Some(values) = props.values.get(key.as_str()) {
            unsafe { *count = values.0.len() as i32 }
            OfxStatus::OK
        } else {
            eprintln!("propGetDimension: {} not found in {}", key, props.name);
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
    let props = paramSet.with_object(|p| {
        p.create_param(OfxStr::from_ptr(paramType), OfxStr::from_ptr(name))
    });
    unsafe { *propertySet = props }
    OfxStatus::OK
}

#[allow(unused_variables)]
extern "C" fn paramGetHandle(
    paramSet: OfxParamSetHandle,
    name: *const c_char,
    param: *mut OfxParamHandle,
    propertySet: *mut OfxPropertySetHandle,
) -> OfxStatus {
    paramSet.with_object(|ps| {
        if let Some(p) = ps.params.get(OfxStr::from_ptr(name).as_str()) {
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
        // Note: not using OfxStr here. String param values are stored
        // as CString and don't need to be UTF-8
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
    handle: *mut c_void,
    messageType: *const c_char,
    messageId: *const c_char,
    format: *const c_char,
) -> OfxStatus {
    println!(
        "\n{}: {}. {}\n",
        OfxStr::from_ptr(messageType),
        if messageId.is_null() {
            OfxStr::from_str("(null)\0")
        } else {
            OfxStr::from_ptr(messageId)
        },
        OfxStr::from_ptr(format)
    );

    // TODO: we're assuming handle is a valid effect instance
    // handle. The spec also allows it to be an effect descriptor
    // handle, or null.
    OfxImageEffectHandle::from(handle).with_object(|effect| {
        // Consume a configured response from the effect instance, or
        // if there are no responses return OK
        effect
            .message_suite_responses
            .pop()
            .unwrap_or(OfxStatus::OK)
    })
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
