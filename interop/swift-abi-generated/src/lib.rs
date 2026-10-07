#![no_std]
#![deny(missing_docs)]
#![doc = "Raw, compiler-derived Swift runtime ABI declarations. This crate is an internal implementation detail and enables no runtime linkage by default."]

#[cfg(all(feature = "apple-runtime", not(target_vendor = "apple")))]
compile_error!("the apple-runtime feature requires an Apple target");

/// Raw Swift runtime exports verified from the installed Apple SDK and Swift compiler.
#[cfg(feature = "apple-runtime")]
pub mod runtime {
    use core::ffi::c_void;

    #[link(name = "swiftCore", kind = "dylib")]
    unsafe extern "C" {
        /// Adds one Swift strong reference and returns the same object pointer.
        ///
        /// The exact `C` ABI signature was emitted by Swift 6.3.3 for an iOS class-reference
        /// operation on both device and simulator targets.
        #[link_name = "swift_retain"]
        pub unsafe fn retain(object: *mut c_void) -> *mut c_void;

        /// Releases one Swift strong reference.
        ///
        /// The exact `C` ABI signature was emitted by Swift 6.3.3 for an iOS class-reference
        /// operation on both device and simulator targets.
        #[link_name = "swift_release"]
        pub unsafe fn release(object: *mut c_void);
    }
}
