#![allow(non_snake_case)]
use crate::types::*;
use std::ffi::{c_char, c_double, c_int, c_uint, c_void};

#[repr(C)]
pub struct OfxImageEffectSuiteV1 {
    pub getPropertySet: extern "C" fn(
        imageEffect: OfxImageEffectHandle,
        propHandle: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    pub getParamSet: extern "C" fn(
        imageEffect: OfxImageEffectHandle,
        paramSet: *mut OfxParamSetHandle,
    ) -> OfxStatus,
    pub clipDefine: extern "C" fn(
        imageEffect: OfxImageEffectHandle,
        name: *const c_char,
        propertySet: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    pub clipGetHandle: extern "C" fn(
        imageEffect: OfxImageEffectHandle,
        name: *const c_char,
        clip: *mut OfxImageClipHandle,
        propertySet: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    pub clipGetPropertySet: extern "C" fn(
        clip: OfxImageClipHandle,
        propHandle: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    pub clipGetImage: extern "C" fn(
        clip: OfxImageClipHandle,
        time: OfxTime,
        region: *const OfxRectD,
        imageHandle: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    pub clipReleaseImage: extern "C" fn(imageHandle: OfxPropertySetHandle) -> OfxStatus,
    pub clipGetRegionOfDefinition: extern "C" fn(
        clip: OfxImageClipHandle,
        time: OfxTime,
        bounds: *mut OfxRectD,
    ) -> OfxStatus,
    pub abort: extern "C" fn(imageEffect: OfxImageEffectHandle) -> c_int,
    pub imageMemoryAlloc: extern "C" fn(
        instanceHandle: OfxImageEffectHandle,
        nBytes: usize,
        memoryHandle: *mut OfxImageMemoryHandle,
    ) -> OfxStatus,
    pub imageMemoryFree: extern "C" fn(memoryHandle: OfxImageMemoryHandle) -> OfxStatus,
    pub imageMemoryLock: extern "C" fn(
        memoryHandle: OfxImageMemoryHandle,
        returnedPtr: *mut *mut c_void,
    ) -> OfxStatus,
    pub imageMemoryUnlock: extern "C" fn(memoryHandle: OfxImageMemoryHandle) -> OfxStatus,
}

#[repr(C)]
pub struct OfxPropertySuiteV1 {
    pub propSetPointer: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut c_void,
    ) -> OfxStatus,
    pub propSetString: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *const c_char,
    ) -> OfxStatus,
    pub propSetDouble: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: c_double,
    ) -> OfxStatus,
    pub propSetInt: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: c_int,
    ) -> OfxStatus,
    pub propSetPointerN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *const *mut c_void,
    ) -> OfxStatus,
    pub propSetStringN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *const *const c_char,
    ) -> OfxStatus,
    pub propSetDoubleN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *const c_double,
    ) -> OfxStatus,
    pub propSetIntN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *const c_int,
    ) -> OfxStatus,
    pub propGetPointer: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut *const c_void,
    ) -> OfxStatus,
    pub propGetString: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut *const c_char,
    ) -> OfxStatus,
    pub propGetDouble: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut c_double,
    ) -> OfxStatus,
    pub propGetInt: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        index: c_int,
        value: *mut c_int,
    ) -> OfxStatus,
    pub propGetPointerN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *mut *const c_void,
    ) -> OfxStatus,
    pub propGetStringN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *mut *const c_char,
    ) -> OfxStatus,
    pub propGetDoubleN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *mut c_double,
    ) -> OfxStatus,
    pub propGetIntN: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: c_int,
        value: *mut c_int,
    ) -> OfxStatus,
    pub propReset: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
    ) -> OfxStatus,
    pub propGetDimension: extern "C" fn(
        properties: OfxPropertySetHandle,
        property: *const c_char,
        count: *mut c_int,
    ) -> OfxStatus,
}

#[repr(C)]
pub struct OfxParameterSuiteV1 {
    pub paramDefine: extern "C" fn(
        paramSet: OfxParamSetHandle,
        paramType: *const c_char,
        name: *const c_char,
        propertySet: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    pub paramGetHandle: extern "C" fn(
        paramSet: OfxParamSetHandle,
        name: *const c_char,
        param: *mut OfxParamHandle,
        propertySet: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    pub paramSetGetPropertySet: extern "C" fn(
        paramSet: OfxParamSetHandle,
        propHandle: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    pub paramGetPropertySet: extern "C" fn(
        paramHandle: OfxParamHandle,
        propHandle: *mut OfxPropertySetHandle,
    ) -> OfxStatus,
    pub paramGetValue:
        unsafe extern "C" fn(paramHandle: OfxParamHandle, ...) -> OfxStatus,
    pub paramGetValueAtTime: unsafe extern "C" fn(
        paramHandle: OfxParamHandle,
        time: OfxTime,
        ...
    ) -> OfxStatus,
    pub paramGetDerivative:
        extern "C" fn(paramHandle: OfxParamHandle, time: OfxTime) -> OfxStatus,
    pub paramGetIntegral: extern "C" fn(
        paramHandle: OfxParamHandle,
        time1: OfxTime,
        time2: OfxTime,
    ) -> OfxStatus,
    pub paramSetValue:
        unsafe extern "C" fn(paramHandle: OfxParamHandle, ...) -> OfxStatus,
    pub paramSetValueAtTime: unsafe extern "C" fn(
        paramHandle: OfxParamHandle,
        time: OfxTime, // time in frames
        ...
    ) -> OfxStatus,
    pub paramGetNumKeys: extern "C" fn(
        paramHandle: OfxParamHandle,
        numberOfKeys: *mut c_uint,
    ) -> OfxStatus,
    pub paramGetKeyTime: extern "C" fn(
        paramHandle: OfxParamHandle,
        nthKey: c_uint,
        time: *mut OfxTime,
    ) -> OfxStatus,
    pub paramGetKeyIndex: extern "C" fn(
        paramHandle: OfxParamHandle,
        time: OfxTime,
        direction: c_int,
        index: *mut c_int,
    ) -> OfxStatus,
    pub paramDeleteKey:
        extern "C" fn(paramHandle: OfxParamHandle, time: OfxTime) -> OfxStatus,
    pub paramDeleteAllKeys: extern "C" fn(paramHandle: OfxParamHandle) -> OfxStatus,
    pub paramCopy: extern "C" fn(
        paramTo: OfxParamHandle,
        paramFrom: OfxParamHandle,
        dstOffset: OfxTime,
        frameRange: *const OfxRangeD,
    ) -> OfxStatus,
    pub paramEditBegin:
        extern "C" fn(paramSet: OfxParamSetHandle, name: *const c_char) -> OfxStatus,
    pub paramEditEnd: extern "C" fn(paramSet: OfxParamSetHandle) -> OfxStatus,
}

#[repr(C)]
pub struct OfxMessageSuiteV1 {
    pub message: extern "C" fn(
        handle: *mut c_void,
        messageType: *const c_char,
        messageId: *const c_char,
        format: *const c_char,
    ) -> OfxStatus,
}

#[repr(C)]
pub struct OfxMemorySuiteV1 {
    pub memoryAlloc: extern "C" fn(
        handle: *mut c_void,
        nBytes: usize,
        allocatedData: *mut *mut c_void,
    ) -> OfxStatus,
    pub memoryFree: extern "C" fn(allocatedData: *mut c_void) -> OfxStatus,
}

pub type OfxThreadFunctionV1 = extern "C" fn(
    threadIndex: c_uint,
    threadMax: c_uint,
    customArg: *mut c_void,
) -> OfxStatus;

#[repr(C)]
pub struct OfxMultiThreadSuiteV1 {
    pub multiThread: extern "C" fn(
        func: OfxThreadFunctionV1,
        nThreads: c_uint,
        customArg: *mut c_void,
    ) -> OfxStatus,
    pub multiThreadNumCPUs: extern "C" fn(nCPUs: *mut c_int) -> OfxStatus,
    pub multiThreadIndex: extern "C" fn(threadIndex: *mut c_int) -> OfxStatus,
    pub multiThreadIsSpawnedThread: extern "C" fn() -> c_int,
    pub mutexCreate: extern "C" fn(mutex: OfxMutexHandle, lockCount: c_int) -> OfxStatus,
    pub mutexDestroy: extern "C" fn(mutex: OfxMutexHandle) -> OfxStatus,
    pub mutexLock: extern "C" fn(mutex: OfxMutexHandle) -> OfxStatus,
    pub mutexUnLock: extern "C" fn(mutex: OfxMutexHandle) -> OfxStatus,
    pub mutexTryLock: extern "C" fn(mutex: OfxMutexHandle) -> OfxStatus,
}
