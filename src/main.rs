use std::collections::HashMap;
use std::error::Error;
use std::ffi::{c_char, c_double, c_int, c_uint, c_void, CStr, CString};
use std::fs;

mod suites;
use suites::*;
mod types;
use types::*;

trait Handle {
    type Object;

    fn as_ref(self) -> &'static mut Self::Object
    where
        Self: Sized + Into<*mut c_void>,
    {
        let ptr: *mut c_void = self.into();
        unsafe { &mut *(ptr as *mut Self::Object) }
    }
}

macro_rules! impl_handle {
    ($handle_name: ident, $object_name: ident) => {
        impl Handle for $handle_name {
            type Object = $object_name;
        }
        impl From<&mut $object_name> for $handle_name {
            fn from(obj: &mut $object_name) -> Self {
                Self::from(obj as *mut $object_name as *mut c_void)
            }
        }
    };
}

impl_handle!(OfxImageEffectHandle, OfxImageEffect);
impl_handle!(OfxParamSetHandle, OfxParamSet);
impl_handle!(OfxPropertySetHandle, OfxPropertySet);

#[derive(Default, Debug)]
pub struct OfxParamSet {
    properties: OfxPropertySet,
}

#[derive(Default, Debug)]
pub struct OfxImageEffect {
    properties: OfxPropertySet,
    params: OfxParamSet,
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
#[derive(Debug)]
#[allow(dead_code)]
enum PropertyValue {
    Pointer(*mut c_void),
    String(CString),
    Double(f64),
    Int(c_int),
    Unset,
}

// Basic conversions
impl From<CString> for PropertyValue {
    fn from(s: CString) -> Self {
        PropertyValue::String(s)
    }
}

impl From<&str> for PropertyValue {
    fn from(s: &str) -> Self {
        PropertyValue::String(CString::new(s).unwrap())
    }
}

impl From<c_int> for PropertyValue {
    fn from(i: c_int) -> Self {
        PropertyValue::Int(i)
    }
}

// OFX uses integers with 0/1 value for boolean properties
impl From<bool> for PropertyValue {
    fn from(b: bool) -> Self {
        PropertyValue::Int(if b { 1 } else { 0 })
    }
}

impl From<f64> for PropertyValue {
    fn from(i: f64) -> Self {
        PropertyValue::Double(i)
    }
}

impl From<*mut c_void> for PropertyValue {
    fn from(i: *mut c_void) -> Self {
        PropertyValue::Pointer(i)
    }
}

#[derive(Default, Debug)]
struct Property(Vec<PropertyValue>);

// Make a PropertyValue from a single value
impl<A: Into<PropertyValue>> From<A> for Property {
    fn from(a: A) -> Self {
        Property([a.into()].into())
    }
}

// Make a PropertyValue from an array of values
impl<T: Copy, const S: usize> From<[T; S]> for Property
where
    PropertyValue: From<T>,
{
    fn from(a: [T; S]) -> Self {
        Property(a.into_iter().map(PropertyValue::from).collect())
    }
}

impl<T: Copy> From<Vec<T>> for Property
where
    PropertyValue: From<T>,
{
    fn from(vec: Vec<T>) -> Self {
        Property(vec.into_iter().map(PropertyValue::from).collect())
    }
}

#[derive(Default, Debug)]
pub struct OfxPropertySet(HashMap<String, Property>);

impl<const S: usize> From<[(&str, Property); S]> for OfxPropertySet {
    fn from(slice: [(&str, Property); S]) -> Self {
        let mut map = HashMap::new();
        for (name, value) in slice {
            map.insert(name.into(), value);
        }
        Self(map)
    }
}

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

const MEMORY_SUITE: OfxMemorySuiteV1 = OfxMemorySuiteV1 {
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
    bundle_path.join("Contents/Info.plist")
}

struct OfxBundle {
    path: std::path::PathBuf,
    plist: plist::Value,
}

fn make_bundle(path: std::path::PathBuf) -> Result<OfxBundle, Box<dyn Error>> {
    let plist = plist::Value::from_file(plist_path(&path))?;
    Ok(OfxBundle { path, plist })
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
        bundle.path.join("Contents/Linux-x86-64").join(lib)
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
                        return make_bundle(path).ok();
                    }
                }
            }
            None
        });
        return x.collect();
    }
    Vec::new()
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
    const VERSION_NAME: &str = env!("CARGO_PKG_VERSION");
    let version: Vec<_> = VERSION_NAME
        .split('.')
        .map(|s| s.parse::<c_int>().unwrap())
        .collect();
    let mut host_props = OfxPropertySet::from([
        ("OfxPropName", "openfx-driver".into()),
        ("OfxPropLabel", "OpenFX Driver".into()),
        ("OfxPropVersion", version.into()),
        ("OfxPropVersionLabel", VERSION_NAME.into()),
        ("OfxPropAPIVersion", [1, 4].into()),
        ("OfxImageEffectHostPropIsBackground", false.into()),
        ("OfxImageEffectPropSupportsOverlays", false.into()),
        ("OfxImageEffectPropSupportsMultiResolution", false.into()),
        ("OfxImageEffectPropSupportsTiles", false.into()),
        ("OfxImageEffectPropTemporalClipAccess", false.into()),
        ("OfxImageEffectPropMultipleClipDepths", false.into()),
        ("OfxImageEffectPropSupportsMultipleClipPARs", false.into()),
        ("OfxImageEffectPropSetableFrameRate", false.into()),
        ("OfxImageEffectPropSetableFielding", false.into()),
        ("OfxImageEffectInstancePropSequentialRender", false.into()),
        ("OfxParamHostPropSupportsStringAnimation", false.into()),
        ("OfxParamHostPropSupportsCustomInteract", false.into()),
        ("OfxParamHostPropSupportsChoiceAnimation", false.into()),
        ("OfxParamHostPropSupportsStrChoiceAnimation", false.into()),
        ("OfxParamHostPropSupportsBooleanAnimation", false.into()),
        ("OfxParamHostPropSupportsCustomAnimation", false.into()),
        ("OfxParamHostPropSupportsParametricAnimation", false.into()),
        // Resolve GPU extensions weirdly use "false"/"true" strings
        ("OfxImageEffectPropOpenCLRenderSupported", "false".into()),
        ("OfxImageEffectPropCudaRenderSupported", "false".into()),
        ("OfxImageEffectPropCudaStreamSupported", "false".into()),
        ("OfxImageEffectPropMetalRenderSupported", "false".into()),
        ("OfxImageEffectPropRenderQualityDraft", false.into()),
        ("OfxParamHostPropMaxParameters", (-1).into()),
        ("OfxParamHostPropMaxPages", 0.into()),
        ("OfxParamHostPropPageRowColumnCount", [0, 0].into()),
        (
            "OfxImageEffectPropSupportedComponents",
            "OfxImageComponentRGBA".into(),
        ),
        (
            "OfxImageEffectPropSupportedContexts",
            "OfxImageEffectContextFilter".into(),
        ),
        (
            "OfxImageEffectPropSupportedPixelDepths",
            "OfxBitDepthFloat".into(),
        ),
    ]);
    let host = OfxHost {
        host: (&mut host_props).into(),
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
                OfxPropertySetHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            );
            let effect: OfxImageEffect = Default::default();
            let stat2 = p.call_action(
                "OfxActionDescribe",
                std::ptr::addr_of!(effect) as *const c_void,
                OfxPropertySetHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            );
            let stat3 = p.call_action(
                "OfxActionUnload",
                std::ptr::null(),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
                OfxPropertySetHandle::from(std::ptr::null_mut()),
            );
            println!(
                "  {:?}, Load returned {:?}, Describe returned {:?}, Unload returned {:?}",
                p, stat, stat2, stat3
            );
            if stat2 == OfxStatus::OK {
                println!("{:?}", effect)
            }
        }
        println!()
    }
}
