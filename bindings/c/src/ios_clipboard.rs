use core::panic::AssertUnwindSafe;
use core::ptr;
use framework_abi::{FrameworkOwnedBuffer, FrameworkStatus, FrameworkStr, catch_unwind_status};

#[cfg(target_os = "ios")]
use alloc::boxed::Box;
#[cfg(target_os = "ios")]
use core::future::Future;
#[cfg(target_os = "ios")]
use core::pin::pin;
#[cfg(target_os = "ios")]
use core::task::{Context, Poll, Waker};

/// Fixed-width clipboard-availability tag.
pub type FrameworkIosClipboardAvailability = u32;
/// No current programmatic-read usability or approval query exists; this does not mean permission
/// is required.
pub const FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNKNOWN: FrameworkIosClipboardAvailability = 0;
/// Clipboard access is available.
pub const FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_AVAILABLE: FrameworkIosClipboardAvailability = 1;
/// Clipboard access is unsupported.
pub const FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNSUPPORTED: FrameworkIosClipboardAvailability = 2;
/// Clipboard access requires permission.
pub const FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_REQUIRES_PERMISSION:
    FrameworkIosClipboardAvailability = 3;
/// Clipboard access requires an entitlement.
pub const FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_REQUIRES_ENTITLEMENT:
    FrameworkIosClipboardAvailability = 4;
/// Clipboard access is temporarily unavailable.
pub const FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_TEMPORARILY_UNAVAILABLE:
    FrameworkIosClipboardAvailability = 5;

/// Unique C handle for one caller-owned iOS clipboard backend.
pub struct FrameworkIosClipboard {
    #[cfg(target_os = "ios")]
    clipboard: framework_sharing::Clipboard<ios_sharing::IosClipboardBackend>,
}

fn valid_span(value: FrameworkStr) -> bool {
    match usize::try_from(value.length()) {
        Ok(length) => {
            length <= isize::MAX as usize
                && if length == 0 {
                    value.data().is_null()
                } else {
                    !value.data().is_null()
                }
        }
        Err(_) => false,
    }
}

unsafe fn input_text<'a>(value: FrameworkStr) -> Result<&'a str, FrameworkStatus> {
    if !valid_span(value) {
        return Err(FrameworkStatus::INVALID_ARGUMENT);
    }
    // SAFETY: The C caller keeps this canonical pointer-length span readable and unchanged through
    // the synchronous call; `valid_span` also rejects lengths beyond Rust's slice limit.
    unsafe { value.as_str() }.ok_or(FrameworkStatus::INVALID_ARGUMENT)
}

unsafe fn initialize_native_code(output: *mut i32) {
    if !output.is_null() {
        // SAFETY: The caller promises that a non-null optional output is writable and aligned.
        unsafe { output.write(0) };
    }
}

#[cfg(target_os = "ios")]
unsafe fn write_native_code(output: *mut i32, value: i32) {
    if !output.is_null() {
        // SAFETY: The caller promises that a non-null optional output is writable and aligned.
        unsafe { output.write(value) };
    }
}

#[cfg(target_os = "ios")]
fn availability_tag(value: framework_core::Availability) -> FrameworkIosClipboardAvailability {
    match value {
        framework_core::Availability::Unknown => FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNKNOWN,
        framework_core::Availability::Available => FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_AVAILABLE,
        framework_core::Availability::Unsupported => {
            FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNSUPPORTED
        }
        framework_core::Availability::RequiresPermission => {
            FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_REQUIRES_PERMISSION
        }
        framework_core::Availability::RequiresEntitlement => {
            FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_REQUIRES_ENTITLEMENT
        }
        framework_core::Availability::TemporarilyUnavailable => {
            FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_TEMPORARILY_UNAVAILABLE
        }
        _ => FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNKNOWN,
    }
}

#[cfg(target_os = "ios")]
fn error_status(value: framework_sharing::ClipboardError) -> FrameworkStatus {
    match value {
        framework_sharing::ClipboardError::Backend(error) => FrameworkStatus::from_error(error),
        _ => FrameworkStatus::INTERNAL_ERROR,
    }
}

#[cfg(target_os = "ios")]
fn native_code(value: framework_sharing::ClipboardError) -> i32 {
    value.platform_code().map_or(0, |code| code.get())
}

/// Creates a clipboard handle bound to the current iOS main thread.
///
/// # Safety
/// `out_clipboard` must name aligned writable storage that does not alias another output. Its
/// current value must not name a live handle. The returned handle is unique and must be destroyed
/// once on the main thread; do not copy it or race calls on it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_clipboard_create(
    out_clipboard: *mut *mut FrameworkIosClipboard,
) -> FrameworkStatus {
    if out_clipboard.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller supplies an empty writable handle slot.
    unsafe { out_clipboard.write(ptr::null_mut()) };
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let Some(main_thread) = ios_runtime::main_thread::MainThread::current() else {
                return FrameworkStatus::UNAVAILABLE;
            };
            let clipboard = framework_sharing::Clipboard::new(
                ios_sharing::IosClipboardBackend::new(main_thread),
            );
            // SAFETY: The output was initialized and is the unique writable handle slot.
            unsafe {
                out_clipboard.write(Box::into_raw(Box::new(FrameworkIosClipboard { clipboard })))
            };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Destroys one clipboard handle and clears its original pointer slot.
///
/// # Safety
/// `clipboard` must be null or the original aligned writable slot returned by create. Destroy only
/// once, on the main thread, after all calls using the handle have ended. A null slot or null handle
/// is a no-op; do not copy or alias a live handle
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_clipboard_destroy(
    clipboard: *mut *mut FrameworkIosClipboard,
) {
    if !clipboard.is_null() {
        #[cfg(target_os = "ios")]
        if ios_runtime::main_thread::MainThread::current().is_some() {
            // SAFETY: The caller provides the original writable handle slot and obeys main-thread use.
            let value = unsafe { clipboard.read() };
            if !value.is_null() {
                // SAFETY: Clear the unique original slot before dropping its allocation.
                unsafe { clipboard.write(ptr::null_mut()) };
                let _ = catch_unwind_status(AssertUnwindSafe(|| {
                    // SAFETY: Create returned this unique Box allocation and the caller did not copy it.
                    unsafe { drop(Box::from_raw(value)) };
                    FrameworkStatus::OK
                }));
            }
        }
    }
}

/// Returns the backend's non-prompting availability value.
///
/// On iOS, `Unknown` means UIKit provides no query for current programmatic-read usability or
/// approval; it does not mean permission is required.
///
/// # Safety
/// `clipboard` must be null or a live handle used only on the main thread. `out_availability`
/// must name aligned writable storage that does not alias the handle or another output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_clipboard_availability(
    clipboard: *const FrameworkIosClipboard,
    out_availability: *mut FrameworkIosClipboardAvailability,
) -> FrameworkStatus {
    if out_availability.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller supplies a writable output slot.
    unsafe { out_availability.write(FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNKNOWN) };
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            if clipboard.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            if ios_runtime::main_thread::MainThread::current().is_none() {
                return FrameworkStatus::UNAVAILABLE;
            }
            // SAFETY: The caller promises a live unique handle used on the main thread.
            let clipboard = unsafe { &*clipboard };
            // SAFETY: The output is valid and does not alias the handle by the function contract.
            unsafe { out_availability.write(availability_tag(clipboard.clipboard.availability())) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = clipboard;
            // SAFETY: The output was validated and initialized above.
            unsafe { out_availability.write(FRAMEWORK_IOS_CLIPBOARD_AVAILABILITY_UNSUPPORTED) };
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Reads an owned plain-text value from the general pasteboard.
///
/// # Safety
/// `clipboard` must be null or a live main-thread handle. Both required outputs must name aligned
/// writable storage, be distinct from each other and the handle, and not alias a live allocation
/// `out_text` must be empty on entry. A non-null optional `out_native_code` must be aligned and
/// writable, and distinct from all other inputs and outputs
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_clipboard_read(
    clipboard: *mut FrameworkIosClipboard,
    out_has_value: *mut u8,
    out_text: *mut FrameworkOwnedBuffer,
    out_native_code: *mut i32,
) -> FrameworkStatus {
    // SAFETY: The caller promises that all non-null output slots are writable and aligned.
    unsafe { initialize_native_code(out_native_code) };
    if !out_has_value.is_null() {
        // SAFETY: The caller promises that a non-null required output is writable and aligned.
        unsafe { out_has_value.write(0) };
    }
    if !out_text.is_null() {
        // SAFETY: The caller promises an empty writable descriptor on entry.
        unsafe { out_text.write(FrameworkOwnedBuffer::default()) };
    }
    if out_has_value.is_null() || out_text.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            if clipboard.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            if ios_runtime::main_thread::MainThread::current().is_none() {
                return FrameworkStatus::UNAVAILABLE;
            }
            // SAFETY: The caller promises a live unique handle used on the main thread.
            let clipboard = unsafe { &mut *clipboard };
            let mut future = pin!(clipboard.clipboard.read());
            let mut context = Context::from_waker(Waker::noop());
            match future.as_mut().poll(&mut context) {
                Poll::Ready(Ok(None)) => FrameworkStatus::OK,
                Poll::Ready(Ok(Some(text))) => {
                    let buffer = match FrameworkOwnedBuffer::try_from_vec(text.into_bytes()) {
                        Ok(buffer) => buffer,
                        Err(_) => return FrameworkStatus::RESOURCE_EXHAUSTED,
                    };
                    // SAFETY: The outputs were validated and are distinct by the function contract.
                    unsafe {
                        out_text.write(buffer);
                        out_has_value.write(1);
                    }
                    FrameworkStatus::OK
                }
                Poll::Ready(Err(error)) => {
                    // SAFETY: The optional output was initialized and is writable by contract.
                    unsafe { write_native_code(out_native_code, native_code(error)) };
                    error_status(error)
                }
                Poll::Pending => FrameworkStatus::INTERNAL_ERROR,
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = clipboard;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Writes exact UTF-8 text to the general pasteboard.
///
/// B6 assigns `UIPasteboard.string`, replacing all current pasteboard items, including any
/// non-text representations.
///
/// # Safety
/// `clipboard` must be null or a live main-thread handle. `text` must be a canonical pointer-length
/// span readable and unchanged through this synchronous call. A non-null optional
/// `out_native_code` must be aligned and writable and must not alias another input or output
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_clipboard_write(
    clipboard: *mut FrameworkIosClipboard,
    text: FrameworkStr,
    out_native_code: *mut i32,
) -> FrameworkStatus {
    // SAFETY: The caller promises that a non-null optional output is writable and aligned.
    unsafe { initialize_native_code(out_native_code) };
    catch_unwind_status(AssertUnwindSafe(|| {
        // SAFETY: The C caller keeps the text span valid through this synchronous call.
        let text = match unsafe { input_text(text) } {
            Ok(value) => value,
            Err(status) => return status,
        };
        #[cfg(target_os = "ios")]
        {
            if clipboard.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            if ios_runtime::main_thread::MainThread::current().is_none() {
                return FrameworkStatus::UNAVAILABLE;
            }
            // SAFETY: The caller promises a live unique handle used on the main thread.
            let clipboard = unsafe { &mut *clipboard };
            let mut future = pin!(clipboard.clipboard.write(text));
            let mut context = Context::from_waker(Waker::noop());
            match future.as_mut().poll(&mut context) {
                Poll::Ready(Ok(())) => FrameworkStatus::OK,
                Poll::Ready(Err(error)) => {
                    // SAFETY: The optional output was initialized and is writable by contract.
                    unsafe { write_native_code(out_native_code, native_code(error)) };
                    error_status(error)
                }
                Poll::Pending => FrameworkStatus::INTERNAL_ERROR,
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = clipboard;
            let _ = text;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Clears all current general-pasteboard items, including non-text representations.
///
/// # Safety
/// `clipboard` must be null or a live main-thread handle. A non-null optional `out_native_code`
/// must be aligned and writable and must not alias another input or output
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_clipboard_clear(
    clipboard: *mut FrameworkIosClipboard,
    out_native_code: *mut i32,
) -> FrameworkStatus {
    // SAFETY: The caller promises that a non-null optional output is writable and aligned.
    unsafe { initialize_native_code(out_native_code) };
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            if clipboard.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            if ios_runtime::main_thread::MainThread::current().is_none() {
                return FrameworkStatus::UNAVAILABLE;
            }
            // SAFETY: The caller promises a live unique handle used on the main thread.
            let clipboard = unsafe { &mut *clipboard };
            let mut future = pin!(clipboard.clipboard.clear());
            let mut context = Context::from_waker(Waker::noop());
            match future.as_mut().poll(&mut context) {
                Poll::Ready(Ok(())) => FrameworkStatus::OK,
                Poll::Ready(Err(error)) => {
                    // SAFETY: The optional output was initialized and is writable by contract.
                    unsafe { write_native_code(out_native_code, native_code(error)) };
                    error_status(error)
                }
                Poll::Pending => FrameworkStatus::INTERNAL_ERROR,
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = clipboard;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
