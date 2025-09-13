use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, Weak};

// ========= Handles =========

// Define our own handle types which wrap the openfx_rs versions.
//
// This allows us to implement pointer conversions, Hash, and Sync.
macro_rules! handle {
    ($name: ident, $ofxname: ident) => {
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(openfx_rs::types::$ofxname);
        impl From<$name> for *mut c_void {
            fn from(handle: $name) -> Self {
                handle.0.into()
            }
        }
        impl From<*mut c_void> for $name {
            fn from(ptr: *mut c_void) -> Self {
                Self(ptr.into())
            }
        }
        impl From<openfx_rs::types::$ofxname> for $name {
            fn from(h: openfx_rs::types::$ofxname) -> Self {
                Self(h)
            }
        }
        impl From<$name> for openfx_rs::types::$ofxname {
            fn from(handle: $name) -> Self {
                handle.0
            }
        }
        unsafe impl Send for $name {}

        impl std::hash::Hash for $name {
            fn hash<H>(&self, state: &mut H)
            where
                H: std::hash::Hasher,
            {
                self.0 .0.hash(state);
            }
        }
    };
}

handle!(ImageClipHandle, OfxImageClipHandle);
handle!(ImageEffectHandle, OfxImageEffectHandle);
handle!(ImageMemoryHandle, OfxImageMemoryHandle);
handle!(ParamHandle, OfxParamHandle);
handle!(ParamSetHandle, OfxParamSetHandle);
handle!(PropertySetHandle, OfxPropertySetHandle);

/// Holder for objects which can cross the API boundary.
///
/// Essentially an Arc<Mutex<T>> with some convenience
/// features.
#[derive(Default)]
pub struct Object<T>(Arc<Mutex<T>>);

impl<T> Object<T> {
    pub fn lock(&self) -> MutexGuard<'_, T> {
        // Locking should never fail since the app is single-threaded
        // for now, so just unwrap.
        self.0.lock().unwrap()
    }
}

impl<T> Clone for Object<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Serialize> Serialize for Object<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.lock().serialize(serializer)
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Object<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(v) = self.0.try_lock() {
            write!(f, "{:?}", v)
        } else {
            write!(f, "Object([locked]{:?})", self.0)
        }
    }
}

pub trait IntoObject: Sized {
    fn into_object(self) -> Object<Self> {
        Object(Arc::new(Mutex::new(self)))
    }
}

/// Keep track of valid handles for a single type.
///
/// Handles are defined in the OFX API as void pointers to opaque
/// objects controlled by the host. Plugins can only access the
/// contents through API functions.
///
/// Here, objects which can be referred to by a handle are stored in
/// an Object<T>. A handle stores the address of the underlying object
/// (which won't move because it's boxed by the Object
/// wrapper). However, to preserve safety, handles are never actually
/// dereferenced. Instead, the HandleManager maintains a map of
/// handles, and Weak pointers to the underlying object. This has
/// several benefits:
/// - Avoids unsafe code
/// - Invalid handles are detected because they don't exist in the map
/// - Handles to dead objects are detected by the Weak pointer
pub struct HandleManager<T, H> {
    handle_to_ptr: HashMap<H, Weak<Mutex<T>>>,
}

impl<T, H> HandleManager<T, H>
where
    H: From<*mut c_void> + Eq + std::hash::Hash + Copy,
{
    pub fn new() -> Self {
        HandleManager {
            handle_to_ptr: HashMap::new(),
        }
    }

    /// Create a handle for an object.
    pub fn get_handle(&mut self, obj: Object<T>) -> H {
        let handle: H = (Arc::as_ptr(&obj.0) as *mut c_void).into();
        self.handle_to_ptr.insert(handle, Arc::downgrade(&obj.0));
        handle
    }
}

/// A trait for handles to OFX objects.
///
/// Provides methods to access the underlying objects referred to by a
/// handle.
pub trait Handle: Sized + Eq + std::hash::Hash + std::fmt::Debug + 'static {
    type Object;
    fn handle_manager() -> &'static LazyLock<Mutex<HandleManager<Self::Object, Self>>>;

    /// Get the underlying object of a handle.
    ///
    /// Panics if the handle is invalid or points to a deallocated
    /// object (these are errors in the plugin and if they occur we
    /// can't reasonably recover, so it's best to fail immediately
    /// with the option of backtrace).
    fn as_arc(&self) -> Object<Self::Object> {
        if let Some(weak) = Self::handle_manager()
            .lock()
            .unwrap()
            .handle_to_ptr
            .get(self)
        {
            Object(weak.upgrade().unwrap_or_else(|| {
                panic!("Handle {:?} points to deallocated object", self)
            }))
        } else {
            panic!("Bad handle {:?}", self);
        }
    }
}

pub trait WithObject<Obj> {
    /// Run a function on the underlying object.
    ///
    /// This uses as_arc() and can panic under the same conditions.
    fn with_object<F, T>(self, callback: F) -> T
    where
        F: FnOnce(&mut Obj) -> T;
}

// Blanket impl for all handles
impl<H> WithObject<H::Object> for H
where
    H: Handle,
{
    fn with_object<F, T>(self, callback: F) -> T
    where
        F: FnOnce(&mut H::Object) -> T,
    {
        let mutex = self.as_arc();
        let guard = &mut mutex.lock();
        callback(guard)
    }
}

pub trait ToHandle: Clone {
    type Handle;
    fn to_handle(&self) -> Self::Handle
    where
        Self::Handle: From<Self>,
    {
        self.clone().into()
    }
}

/// Implement traits for a handle and its associated object: From,
/// Handle, WithObject, ToHandle. Provides convenient conversion
/// between handles and corresponding objects.
macro_rules! impl_handle {
    ($handle_name: ident, $ofx_handle_name: ident, $object_name: ident) => {
        impl Handle for $handle_name {
            type Object = $object_name;
            fn handle_manager(
            ) -> &'static LazyLock<Mutex<HandleManager<Self::Object, Self>>> {
                static MANAGER: LazyLock<
                    Mutex<HandleManager<$object_name, $handle_name>>,
                > = LazyLock::new(|| Mutex::new(HandleManager::new()));
                &MANAGER
            }
        }

        impl From<Object<$object_name>> for $handle_name {
            fn from(obj: Object<$object_name>) -> Self {
                $handle_name::handle_manager()
                    .lock()
                    .unwrap()
                    .get_handle(obj)
            }
        }

        impl ToHandle for Object<$object_name> {
            type Handle = $handle_name;
        }

        // Convert openfx_rs handle to our wrapper, and call
        // with_object on that
        impl WithObject<$object_name> for openfx_rs::types::$ofx_handle_name {
            fn with_object<F, T>(self, callback: F) -> T
            where
                F: FnOnce(&mut $object_name) -> T,
            {
                $handle_name::from(self).with_object(callback)
            }
        }
    };
}
