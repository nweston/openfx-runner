#![allow(non_snake_case)]
use crate::handles::{ToHandle, WithObject};
use crate::{handles::*, Clip};
use crate::{
    log_error, output, FromProperty, OfxError, ParamValue, PropertySet, PropertyValue,
};

#[cfg(target_os = "windows")]
use libc::{free, malloc};
#[cfg(not(target_os = "windows"))]
use libc::{free, posix_memalign};

use openfx_rs::constants;
use openfx_rs::constants::ofxstatus;
use openfx_rs::strings::OfxStr;
use openfx_rs::types::*;
// Import directly from openfx_sys. openfx_rs provides wrappers which
// are convenient for a plugin, but not useful for supplying our own
// suite implementations
use openfx_sys::{
    OfxImageEffectSuiteV1, OfxMemorySuiteV1, OfxMessageSuiteV1, OfxMultiThreadSuiteV1,
    OfxParameterSuiteV1, OfxPropertySuiteV1,
};
use std::collections::HashMap;
use std::ffi::{c_char, c_double, c_int, c_uint, c_void, CStr};

// ========= ImageEffectSuite =========
extern "C" fn getPropertySet(
    imageEffect: openfx_rs::types::OfxImageEffectHandle,
    propHandle: *mut openfx_rs::types::OfxPropertySetHandle,
) -> OfxStatus {
    unsafe {
        *propHandle =
            imageEffect.with_object(|effect| effect.properties.to_handle().into())
    };
    ofxstatus::OK.into()
}

extern "C" fn getParamSet(
    imageEffect: openfx_rs::types::OfxImageEffectHandle,
    paramSet: *mut openfx_rs::types::OfxParamSetHandle,
) -> OfxStatus {
    unsafe {
        *paramSet = imageEffect.with_object(|effect| effect.param_set.to_handle().into())
    };
    ofxstatus::OK.into()
}

extern "C" fn clipDefine(
    imageEffect: openfx_rs::types::OfxImageEffectHandle,
    name: *const c_char,
    propertySet: *mut openfx_rs::types::OfxPropertySetHandle,
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
            *propertySet = props.to_handle().into();
        }
    }
    ofxstatus::OK.into()
}

#[allow(unused_variables)]
extern "C" fn clipGetHandle(
    imageEffect: openfx_rs::types::OfxImageEffectHandle,
    name: *const c_char,
    clip: *mut openfx_rs::types::OfxImageClipHandle,
    propertySet: *mut openfx_rs::types::OfxPropertySetHandle,
) -> OfxStatus {
    imageEffect
        .with_object(|effect| {
            if let Some(c) = effect.clips.get(OfxStr::from_ptr(name).as_str()) {
                unsafe {
                    *clip = c.to_handle().into();
                    if !propertySet.is_null() {
                        *propertySet = c.lock().properties.to_handle().into();
                    }
                }
                ofxstatus::OK
            } else {
                ofxstatus::ErrUnknown
            }
        })
        .into()
}

#[allow(unused_variables)]
extern "C" fn clipGetPropertySet(
    clip: openfx_rs::types::OfxImageClipHandle,
    propHandle: *mut openfx_rs::types::OfxPropertySetHandle,
) -> OfxStatus {
    clip.with_object(|c| {
        let handle = c.properties.to_handle().into();
        unsafe { *propHandle = handle }
    });
    ofxstatus::OK.into()
}

#[allow(unused_variables)]
extern "C" fn clipGetImage(
    clip: openfx_rs::types::OfxImageClipHandle,
    time: OfxTime,
    _region: *const OfxRectD,
    imageHandle: *mut openfx_rs::types::OfxPropertySetHandle,
) -> OfxStatus {
    clip.with_object(|c| {
        if let Some(handle) = c.get_image_handle_at_time(time) {
            unsafe {
                *imageHandle = handle.into();
            }
            ofxstatus::OK
        } else {
            ofxstatus::Failed
        }
    })
    .into()
}

#[allow(unused_variables)]
extern "C" fn clipReleaseImage(
    imageHandle: openfx_rs::types::OfxPropertySetHandle,
) -> OfxStatus {
    Clip::release_image_handle(imageHandle.into());
    ofxstatus::OK.into()
}

#[allow(unused_variables)]
extern "C" fn clipGetRegionOfDefinition(
    clip: openfx_rs::types::OfxImageClipHandle,
    time: OfxTime,
    bounds: *mut OfxRectD,
) -> OfxStatus {
    clip.with_object(|c| {
        if let Some(rod) = c.region_of_definition {
            unsafe {
                *bounds = rod;
            }
            ofxstatus::OK
        } else {
            ofxstatus::Failed
        }
    })
    .into()
}

#[allow(unused_variables)]
extern "C" fn abort(imageEffect: openfx_rs::types::OfxImageEffectHandle) -> c_int {
    return 0;
}

#[allow(unused_variables)]
extern "C" fn imageMemoryAlloc(
    instanceHandle: openfx_rs::types::OfxImageEffectHandle,
    nBytes: usize,
    memoryHandle: *mut openfx_rs::types::OfxImageMemoryHandle,
) -> OfxStatus {
    unsafe {
        // Allocate memory directly and return the pointer as a
        // handle.

        // 16-byte alignment is required by the spec, but Windows
        // doesn't have posix_memalign so use regular malloc for now
        #[cfg(target_os = "windows")]
        {
            let ptr: *mut c_void = malloc(nBytes);
            if ptr.is_null() {
                return OfxStatus::ErrMemory;
            }
            *memoryHandle = ptr.into();
        }

        #[cfg(not(target_os = "windows"))]
        {
            let mut ptr: *mut c_void = std::ptr::null_mut();
            if posix_memalign(&mut ptr, 16, nBytes) != 0 {
                return ofxstatus::ErrMemory.into();
            }
            *memoryHandle = openfx_rs::types::OfxImageMemoryHandle(ptr as _);
        }
    };
    ofxstatus::OK.into()
}

#[allow(unused_variables)]
extern "C" fn imageMemoryFree(
    memoryHandle: openfx_rs::types::OfxImageMemoryHandle,
) -> OfxStatus {
    unsafe {
        free(memoryHandle.0 as _);
    };
    ofxstatus::OK.into()
}

#[allow(unused_variables)]
extern "C" fn imageMemoryLock(
    memoryHandle: openfx_rs::types::OfxImageMemoryHandle,
    returnedPtr: *mut *mut c_void,
) -> OfxStatus {
    // The handle is already a pointer to allocated memory, just
    // return it
    unsafe {
        *returnedPtr = memoryHandle.0 as _;
    }
    ofxstatus::OK.into()
}
#[allow(unused_variables)]
extern "C" fn imageMemoryUnlock(
    memoryHandle: openfx_rs::types::OfxImageMemoryHandle,
) -> OfxStatus {
    // Nothing to do
    ofxstatus::OK.into()
}

pub const IMAGE_EFFECT_SUITE: OfxImageEffectSuiteV1 = OfxImageEffectSuiteV1 {
    getPropertySet: Some(getPropertySet),
    getParamSet: Some(getParamSet),
    clipDefine: Some(clipDefine),
    clipGetHandle: Some(clipGetHandle),
    clipGetPropertySet: Some(clipGetPropertySet),
    clipGetImage: Some(clipGetImage),
    clipReleaseImage: Some(clipReleaseImage),
    clipGetRegionOfDefinition: Some(clipGetRegionOfDefinition),
    abort: Some(abort),
    imageMemoryAlloc: Some(imageMemoryAlloc),
    imageMemoryFree: Some(imageMemoryFree),
    imageMemoryLock: Some(imageMemoryLock),
    imageMemoryUnlock: Some(imageMemoryUnlock),
};

// ========= Property Suite =========
fn set_property(
    properties: openfx_rs::types::OfxPropertySetHandle,
    name: *const c_char,
    index: c_int,
    value: PropertyValue,
) -> OfxStatus {
    properties.with_object(|props| {
        props.set(OfxStr::from_ptr(name).as_str(), index as usize, value)
    });
    ofxstatus::OK.into()
}

fn set_property_n<T: Into<PropertyValue> + Copy>(
    properties: openfx_rs::types::OfxPropertySetHandle,
    name: *const c_char,
    count: c_int,
    value: *const T,
) -> OfxStatus {
    let s = unsafe { std::slice::from_raw_parts(value, count as usize) };
    for (i, v) in s.iter().enumerate() {
        set_property(properties, name, i as i32, (*v).into());
    }
    ofxstatus::OK.into()
}

extern "C" fn propSetPointer(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut c_void,
) -> OfxStatus {
    set_property(properties, property, index, value.into())
}

extern "C" fn propSetString(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *const c_char,
) -> OfxStatus {
    set_property(properties, property, index, value.into())
}

extern "C" fn propSetDouble(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: c_double,
) -> OfxStatus {
    set_property(properties, property, index, value.into())
}

extern "C" fn propSetInt(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: c_int,
) -> OfxStatus {
    set_property(properties, property, index, value.into())
}

#[allow(unused_variables)]
extern "C" fn propSetPointerN(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const *mut c_void,
) -> OfxStatus {
    set_property_n(properties, property, count, value)
}

#[allow(unused_variables)]
extern "C" fn propSetStringN(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const *const c_char,
) -> OfxStatus {
    set_property_n(properties, property, count, value)
}

#[allow(unused_variables)]
extern "C" fn propSetDoubleN(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *const c_double,
) -> OfxStatus {
    set_property_n(properties, property, count, value)
}

#[allow(unused_variables)]
extern "C" fn propSetIntN(
    properties: openfx_rs::types::OfxPropertySetHandle,
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
                    status: ofxstatus::ErrUnknown,
                }),
                _ => Err(OfxError {
                    message: format!(
                        "{} {} unexpected type: {:?} in {}",
                        key, index, p, props.name
                    ),
                    status: ofxstatus::ErrUnknown,
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
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut *mut c_void,
) -> OfxStatus {
    properties
        .with_object(|props| {
            get_property(value, props, OfxStr::from_ptr(property), index as usize)
                .check_status("propGetPointer: ")
        })
        .into()
}

extern "C" fn propGetString(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut *mut c_char,
) -> OfxStatus {
    properties
        .with_object(|props| {
            get_property(value, props, OfxStr::from_ptr(property), index as usize)
                .check_status("propGetString: ")
        })
        .into()
}

extern "C" fn propGetDouble(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut c_double,
) -> OfxStatus {
    properties
        .with_object(|props| {
            get_property(value, props, OfxStr::from_ptr(property), index as usize)
                .check_status("propGetDouble: ")
        })
        .into()
}

extern "C" fn propGetInt(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    index: c_int,
    value: *mut c_int,
) -> OfxStatus {
    properties
        .with_object(|props| {
            get_property(value, props, OfxStr::from_ptr(property), index as usize)
                .check_status("propGetInt: ")
        })
        .into()
}

#[allow(unused_variables)]
extern "C" fn propGetPointerN(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut *mut c_void,
) -> OfxStatus {
    properties
        .with_object(|props| {
            get_property_array(value, props, OfxStr::from_ptr(property), count as usize)
                .check_status("propGetPointerN: ")
        })
        .into()
}

#[allow(unused_variables)]
extern "C" fn propGetStringN(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut *mut c_char,
) -> OfxStatus {
    properties
        .with_object(|props| {
            get_property_array(value, props, OfxStr::from_ptr(property), count as usize)
                .check_status("propGetStringN: ")
        })
        .into()
}

#[allow(unused_variables)]
extern "C" fn propGetDoubleN(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut c_double,
) -> OfxStatus {
    properties
        .with_object(|props| {
            get_property_array(value, props, OfxStr::from_ptr(property), count as usize)
                .check_status("propGetDoubleN: ")
        })
        .into()
}

#[allow(unused_variables)]
extern "C" fn propGetIntN(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    count: c_int,
    value: *mut c_int,
) -> OfxStatus {
    properties
        .with_object(|props| {
            get_property_array(value, props, OfxStr::from_ptr(property), count as usize)
                .check_status("propGetIntN: ")
        })
        .into()
}

#[allow(unused_variables)]
extern "C" fn propReset(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
) -> OfxStatus {
    panic!("Not implemented!");
}

extern "C" fn propGetDimension(
    properties: openfx_rs::types::OfxPropertySetHandle,
    property: *const c_char,
    count: *mut c_int,
) -> OfxStatus {
    let key = OfxStr::from_ptr(property);
    properties
        .with_object(|props| {
            if let Some(values) = props.values.get(key.as_str()) {
                unsafe { *count = values.0.len() as i32 }
                ofxstatus::OK
            } else {
                log_error!("propGetDimension: {} not found in {}", key, props.name);
                ofxstatus::ErrUnknown
            }
        })
        .into()
}

pub const PROPERTY_SUITE: OfxPropertySuiteV1 = OfxPropertySuiteV1 {
    propSetPointer: Some(propSetPointer),
    propSetString: Some(propSetString),
    propSetDouble: Some(propSetDouble),
    propSetInt: Some(propSetInt),
    propSetPointerN: Some(propSetPointerN),
    propSetStringN: Some(propSetStringN),
    propSetDoubleN: Some(propSetDoubleN),
    propSetIntN: Some(propSetIntN),
    propGetPointer: Some(propGetPointer),
    propGetString: Some(propGetString),
    propGetDouble: Some(propGetDouble),
    propGetInt: Some(propGetInt),
    propGetPointerN: Some(propGetPointerN),
    propGetStringN: Some(propGetStringN),
    propGetDoubleN: Some(propGetDoubleN),
    propGetIntN: Some(propGetIntN),
    propReset: Some(propReset),
    propGetDimension: Some(propGetDimension),
};

// ========= Parameter suite =========
#[allow(unused_variables)]
extern "C" fn paramDefine(
    paramSet: openfx_rs::types::OfxParamSetHandle,
    paramType: *const c_char,
    name: *const c_char,
    propertySet: *mut openfx_rs::types::OfxPropertySetHandle,
) -> OfxStatus {
    let props = paramSet.with_object(|p| {
        p.create_param(OfxStr::from_ptr(paramType), OfxStr::from_ptr(name))
    });
    unsafe { *propertySet = props.into() }
    ofxstatus::OK.into()
}

#[allow(unused_variables)]
extern "C" fn paramGetHandle(
    paramSet: openfx_rs::types::OfxParamSetHandle,
    name: *const c_char,
    param: *mut openfx_rs::types::OfxParamHandle,
    propertySet: *mut openfx_rs::types::OfxPropertySetHandle,
) -> OfxStatus {
    paramSet
        .with_object(|ps| {
            if let Some(p) = ps.params.get(OfxStr::from_ptr(name).as_str()) {
                unsafe {
                    *param = p.to_handle().into();
                    if !propertySet.is_null() {
                        *propertySet = p.lock().properties.to_handle().into();
                    }
                }
                ofxstatus::OK
            } else {
                ofxstatus::ErrUnknown
            }
        })
        .into()
}

extern "C" fn paramSetGetPropertySet(
    paramSet: openfx_rs::types::OfxParamSetHandle,
    propHandle: *mut openfx_rs::types::OfxPropertySetHandle,
) -> OfxStatus {
    unsafe { *propHandle = paramSet.with_object(|p| p.properties.to_handle().into()) };
    ofxstatus::OK.into()
}

#[allow(unused_variables)]
extern "C" fn paramGetPropertySet(
    paramHandle: openfx_rs::types::OfxParamHandle,
    propHandle: *mut openfx_rs::types::OfxPropertySetHandle,
) -> OfxStatus {
    paramHandle.with_object(|param| unsafe {
        *propHandle = param.properties.to_handle().into();
    });
    ofxstatus::OK.into()
}

unsafe extern "C" {
    fn paramGetValue(paramHandle: openfx_rs::types::OfxParamHandle, ...) -> OfxStatus;
    fn paramGetValueAtTime(
        paramHandle: openfx_rs::types::OfxParamHandle,
        time: OfxTime,
        ...
    ) -> OfxStatus;
    fn paramSetValue(paramHandle: openfx_rs::types::OfxParamHandle, ...) -> OfxStatus;
    fn paramSetValueAtTime(
        paramHandle: openfx_rs::types::OfxParamHandle,
        time: OfxTime,
        ...
    ) -> OfxStatus;
}

#[unsafe(no_mangle)]
pub extern "C" fn param_value_count(
    paramHandle: openfx_rs::types::OfxParamHandle,
) -> c_int {
    use ParamValue::*;
    paramHandle.with_object(|p| match p.value {
        Double2D(..) | Integer2D(..) => 2,
        Rgb { .. } | Double3D(..) | Integer3D(..) => 3,
        Rgba { .. } => 4,
        Boolean(_) | Choice(_) | Custom(_) | Double(_) | Integer(_) | String(_) => 1,
        Group | Page | Parametric | PushButton => 0,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn param_get_value_1(
    paramHandle: openfx_rs::types::OfxParamHandle,
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
    ofxstatus::OK.into()
}

#[unsafe(no_mangle)]
#[allow(unused_variables)]
pub extern "C" fn param_get_value_2(
    paramHandle: openfx_rs::types::OfxParamHandle,
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
    ofxstatus::OK.into()
}

#[unsafe(no_mangle)]
pub extern "C" fn param_get_value_3(
    paramHandle: openfx_rs::types::OfxParamHandle,
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
    ofxstatus::OK.into()
}

#[unsafe(no_mangle)]
pub extern "C" fn param_get_value_4(
    paramHandle: openfx_rs::types::OfxParamHandle,
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
    ofxstatus::OK.into()
}

#[unsafe(no_mangle)]
pub extern "C" fn param_get_type(
    handle: openfx_rs::types::OfxParamHandle,
) -> *const c_char {
    handle.with_object(|p| {
        if let Ok(PropertyValue::String(s)) =
            p.properties.lock().get(constants::ParamPropType, 0)
        {
            s.as_c_str().as_ptr()
        } else {
            panic!("OfxParamPropType not found on param")
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn param_set_value_boolean(
    handle: openfx_rs::types::OfxParamHandle,
    value: i32,
) {
    handle.with_object(|p| p.value = ParamValue::Boolean(value != 0));
}

#[unsafe(no_mangle)]
pub extern "C" fn param_set_value_integer(
    handle: openfx_rs::types::OfxParamHandle,
    value: i32,
) {
    handle.with_object(|p| p.value = ParamValue::Integer(value));
}

#[unsafe(no_mangle)]
pub extern "C" fn param_set_value_choice(
    handle: openfx_rs::types::OfxParamHandle,
    value: i32,
) {
    handle.with_object(|p| p.value = ParamValue::Choice(value as usize));
}

#[unsafe(no_mangle)]
pub extern "C" fn param_set_value_double(
    handle: openfx_rs::types::OfxParamHandle,
    value: f64,
) {
    handle.with_object(|p| p.value = ParamValue::Double(value));
}

#[unsafe(no_mangle)]
pub extern "C" fn param_set_value_string(
    handle: openfx_rs::types::OfxParamHandle,
    value: *const c_char,
) {
    handle.with_object(|p| {
        // Note: not using OfxStr here. String param values are stored
        // as CString and don't need to be UTF-8
        p.value = ParamValue::String(unsafe { CStr::from_ptr(value) }.into())
    });
}

#[allow(unused_variables)]
extern "C" fn paramGetNumKeys(
    paramHandle: openfx_rs::types::OfxParamHandle,
    numberOfKeys: *mut c_uint,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn paramGetKeyTime(
    paramHandle: openfx_rs::types::OfxParamHandle,
    nthKey: c_uint,
    time: *mut OfxTime,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn paramGetKeyIndex(
    paramHandle: openfx_rs::types::OfxParamHandle,
    time: OfxTime,
    direction: c_int,
    index: *mut c_int,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn paramDeleteKey(
    paramHandle: openfx_rs::types::OfxParamHandle,
    time: OfxTime,
) -> OfxStatus {
    ofxstatus::OK.into()
}

#[allow(unused_variables)]
extern "C" fn paramDeleteAllKeys(
    paramHandle: openfx_rs::types::OfxParamHandle,
) -> OfxStatus {
    ofxstatus::OK.into()
}

#[allow(unused_variables)]
extern "C" fn paramCopy(
    paramTo: openfx_rs::types::OfxParamHandle,
    paramFrom: openfx_rs::types::OfxParamHandle,
    dstOffset: OfxTime,
    frameRange: *const OfxRangeD,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn paramEditBegin(
    paramSet: openfx_rs::types::OfxParamSetHandle,
    name: *const c_char,
) -> OfxStatus {
    ofxstatus::OK.into()
}

#[allow(unused_variables)]
extern "C" fn paramEditEnd(paramSet: openfx_rs::types::OfxParamSetHandle) -> OfxStatus {
    ofxstatus::OK.into()
}

pub const PARAMETER_SUITE: OfxParameterSuiteV1 = OfxParameterSuiteV1 {
    paramDefine: Some(paramDefine),
    paramGetHandle: Some(paramGetHandle),
    paramSetGetPropertySet: Some(paramSetGetPropertySet),
    paramGetPropertySet: Some(paramGetPropertySet),
    paramGetValue: Some(paramGetValue),
    paramGetValueAtTime: Some(paramGetValueAtTime),
    paramGetDerivative: None,
    paramGetIntegral: None,
    paramSetValue: Some(paramSetValue),
    paramSetValueAtTime: Some(paramSetValueAtTime),
    paramGetNumKeys: Some(paramGetNumKeys),
    paramGetKeyTime: Some(paramGetKeyTime),
    paramGetKeyIndex: Some(paramGetKeyIndex),
    paramDeleteKey: Some(paramDeleteKey),
    paramDeleteAllKeys: Some(paramDeleteAllKeys),
    paramCopy: Some(paramCopy),
    paramEditBegin: Some(paramEditBegin),
    paramEditEnd: Some(paramEditEnd),
};

// ========= MessageSuiteV1 =========
unsafe extern "C" {
    unsafe fn message(
        handle: *mut c_void,
        messageType: *const c_char,
        messageId: *const c_char,
        format: *const c_char,
        ...
    ) -> OfxStatus;
}

#[unsafe(no_mangle)]
extern "C" fn message_impl(
    handle: *mut c_void,
    messageType: *const c_char,
    messageId: *const c_char,
    message: *const c_char,
) -> OfxStatus {
    let id_str = if messageId.is_null() {
        OfxStr::from_str("(null)\0")
    } else {
        OfxStr::from_ptr(messageId)
    };
    output!(
        "{}",
        serde_json::to_string(&HashMap::from([
            ("message_type", OfxStr::from_ptr(messageType).as_str(),),
            ("message_id", id_str.as_str()),
            ("message", OfxStr::from_ptr(message).as_str())
        ]))
        .unwrap()
    );

    // TODO: we're assuming handle is a valid effect instance
    // handle. The spec also allows it to be an effect descriptor
    // handle, or null.
    ImageEffectHandle::from(handle)
        .with_object(|effect| {
            // Consume a configured response from the effect instance, or
            // if there are no responses return OK
            effect
                .message_suite_responses
                .pop()
                .unwrap_or(ofxstatus::OK)
        })
        .into()
}

pub const MESSAGE_SUITE: OfxMessageSuiteV1 = OfxMessageSuiteV1 {
    message: Some(message),
};

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
    memoryAlloc: Some(memoryAlloc),
    memoryFree: Some(memoryFree),
};

// ========= Multithread suite =========

#[allow(unused_variables)]
extern "C" fn multiThread(
    func: openfx_sys::OfxThreadFunctionV1,
    nThreads: c_uint,
    customArg: *mut c_void,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn multiThreadNumCPUs(nCPUs: *mut u32) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn multiThreadIndex(threadIndex: *mut u32) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn multiThreadIsSpawnedThread() -> c_int {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn mutexCreate(
    mutex: *mut openfx_sys::OfxMutexHandle,
    lockCount: c_int,
) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn mutexDestroy(mutex: openfx_sys::OfxMutexHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn mutexLock(mutex: openfx_sys::OfxMutexHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn mutexUnLock(mutex: openfx_sys::OfxMutexHandle) -> OfxStatus {
    panic!("Not implemented!")
}

#[allow(unused_variables)]
extern "C" fn mutexTryLock(mutex: openfx_sys::OfxMutexHandle) -> OfxStatus {
    panic!("Not implemented!")
}

pub const MULTI_THREAD_SUITE: OfxMultiThreadSuiteV1 = OfxMultiThreadSuiteV1 {
    multiThread: Some(multiThread),
    multiThreadNumCPUs: Some(multiThreadNumCPUs),
    multiThreadIndex: Some(multiThreadIndex),
    multiThreadIsSpawnedThread: Some(multiThreadIsSpawnedThread),
    mutexCreate: Some(mutexCreate),
    mutexDestroy: Some(mutexDestroy),
    mutexLock: Some(mutexLock),
    mutexUnLock: Some(mutexUnLock),
    mutexTryLock: Some(mutexTryLock),
};
