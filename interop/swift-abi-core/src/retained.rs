use core::ffi::c_void;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ptr::NonNull;

/// Marker for a type whose pointers are native Swift class references.
///
/// # Safety
///
/// Implementors must represent a Swift class reference accepted by the installed Swift runtime's
/// `swift_retain` and `swift_release` operations. This trait does not assert `Send` or `Sync`.
pub unsafe trait SwiftClass {}

/// Opaque marker for a Swift object when its concrete class type is not exposed to Rust.
pub struct SwiftObject {
    _private: [u8; 0],
}

unsafe impl SwiftClass for SwiftObject {}

/// An owned strong reference to a Swift class instance.
///
/// Cloning retains the native object; dropping releases it. The wrapper is intentionally
/// capability-scoped: it exists only when the `apple-runtime` feature is enabled and never adds a
/// Rust reference-count layer.
pub struct SwiftRetained<T: SwiftClass = SwiftObject> {
    pointer: NonNull<T>,
    _type: PhantomData<fn() -> T>,
    _thread_affinity: PhantomData<*mut ()>,
}

impl<T: SwiftClass> SwiftRetained<T> {
    /// Takes ownership of a Swift class reference with one existing strong retain.
    ///
    /// # Safety
    ///
    /// `pointer` must be null or a valid Swift class reference with one strong retain transferred
    /// to this wrapper. The concrete type represented by `T` must match the object. A non-null
    /// pointer is released exactly once when the returned wrapper is dropped.
    pub unsafe fn from_owned_ptr(pointer: *mut T) -> Option<Self> {
        let pointer = NonNull::new(pointer)?;
        Some(Self {
            pointer,
            _type: PhantomData,
            _thread_affinity: PhantomData,
        })
    }

    /// Retains a borrowed Swift class reference and returns an owned wrapper.
    ///
    /// # Safety
    ///
    /// A non-null `pointer` must be a live Swift class reference of type `T` for the duration of
    /// the retain call. The pointer must not be concurrently destroyed while this method retains
    /// it.
    pub unsafe fn retain_borrowed(pointer: *mut T) -> Option<Self> {
        let pointer = NonNull::new(pointer)?;
        unsafe {
            swift_abi_generated::runtime::retain(pointer.as_ptr().cast::<c_void>());
        }
        Some(Self {
            pointer,
            _type: PhantomData,
            _thread_affinity: PhantomData,
        })
    }

    /// Returns a borrowed raw pointer valid while `self` remains alive.
    pub const fn as_ptr(&self) -> *mut T {
        self.pointer.as_ptr()
    }

    /// Transfers the owned strong reference to the caller without releasing it.
    pub fn into_raw(self) -> *mut T {
        let this = ManuallyDrop::new(self);
        this.pointer.as_ptr()
    }
}

impl<T: SwiftClass> Clone for SwiftRetained<T> {
    fn clone(&self) -> Self {
        unsafe {
            swift_abi_generated::runtime::retain(self.pointer.as_ptr().cast::<c_void>());
        }
        Self {
            pointer: self.pointer,
            _type: PhantomData,
            _thread_affinity: PhantomData,
        }
    }
}

impl<T: SwiftClass> Drop for SwiftRetained<T> {
    fn drop(&mut self) {
        unsafe {
            swift_abi_generated::runtime::release(self.pointer.as_ptr().cast::<c_void>());
        }
    }
}
