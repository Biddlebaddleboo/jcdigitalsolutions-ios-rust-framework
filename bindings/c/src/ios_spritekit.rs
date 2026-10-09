use core::panic::AssertUnwindSafe;
use core::ptr;
use framework_abi::{FrameworkStatus, catch_unwind_status};

#[cfg(target_os = "ios")]
use alloc::boxed::Box;
#[cfg(target_os = "ios")]
use framework_spritekit::{SpriteNodePosition, SpriteNodePositionBackend};
#[cfg(target_os = "ios")]
use ios_runtime::main_thread::MainThread;
#[cfg(target_os = "ios")]
use ios_spritekit::IosSpriteNode;

/// One opaque, uniquely owned detached SpriteKit node handle.
#[repr(C)]
pub struct FrameworkIosSpriteKitNode {
    #[cfg(target_os = "ios")]
    node: IosSpriteNode,
    #[cfg(not(target_os = "ios"))]
    _opaque: [u8; 0],
}

/// Creates a detached node with a finite parent-local position.
///
/// # Safety
/// `out_node` must point to aligned writable storage that does not alias a live handle. On iOS,
/// call on the main thread. On success, the caller owns one unique handle and must not copy it,
/// race an operation, or destroy an alias.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_spritekit_node_create(
    x: f64,
    y: f64,
    out_node: *mut *mut FrameworkIosSpriteKitNode,
) -> FrameworkStatus {
    if out_node.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    // SAFETY: The caller promises one aligned writable output slot.
    unsafe { out_node.write(ptr::null_mut()) };
    if !x.is_finite() || !y.is_finite() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let Some(main_thread) = MainThread::current() else {
                return FrameworkStatus::UNAVAILABLE;
            };
            let position = match SpriteNodePosition::new(x, y) {
                Ok(position) => position,
                Err(error) => return FrameworkStatus::from_error(error),
            };
            let node = match IosSpriteNode::new(main_thread, position) {
                Ok(node) => node,
                Err(error) => return FrameworkStatus::from_error(error),
            };
            let handle = Box::new(FrameworkIosSpriteKitNode { node });
            // SAFETY: `out_node` was initialized and remains writable for this call.
            unsafe { out_node.write(Box::into_raw(handle)) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            FrameworkStatus::UNSUPPORTED
        }
    }))
}

/// Reads one node's parent-local position.
///
/// # Safety
/// On iOS, `node` must be a live handle returned by create and this call must run on the main
/// thread. Calls on one handle must be serialized and must not race set or destroy. Each non-null
/// output must point to aligned writable `double` storage; the outputs must be distinct and must
/// not alias the handle. Both outputs are required.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_spritekit_node_get_position(
    node: *const FrameworkIosSpriteKitNode,
    out_x: *mut f64,
    out_y: *mut f64,
) -> FrameworkStatus {
    // SAFETY: The caller promises each non-null output is writable.
    unsafe {
        if !out_x.is_null() {
            out_x.write(0.0);
        }
        if !out_y.is_null() {
            out_y.write(0.0);
        }
    }
    if out_x.is_null() || out_y.is_null() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            if MainThread::current().is_none() {
                return FrameworkStatus::UNAVAILABLE;
            }
            if node.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            // SAFETY: The caller keeps the unique live handle on the main thread for this call.
            let node = unsafe { &*node };
            match node.node.position() {
                Ok(position) => {
                    // SAFETY: Required outputs are writable and distinct by the caller contract.
                    unsafe {
                        out_x.write(position.x());
                        out_y.write(position.y());
                    }
                    FrameworkStatus::OK
                }
                Err(error) => FrameworkStatus::from_error(error),
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            if node.is_null() {
                FrameworkStatus::INVALID_ARGUMENT
            } else {
                FrameworkStatus::UNSUPPORTED
            }
        }
    }))
}

/// Replaces one node's finite parent-local position.
///
/// # Safety
/// On iOS, `node` must be a live uniquely owned handle returned by create and this call must run
/// on the main thread. Calls on one handle must be serialized and must not race get or destroy.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_spritekit_node_set_position(
    node: *mut FrameworkIosSpriteKitNode,
    x: f64,
    y: f64,
) -> FrameworkStatus {
    if !x.is_finite() || !y.is_finite() {
        return FrameworkStatus::INVALID_ARGUMENT;
    }
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            if MainThread::current().is_none() {
                return FrameworkStatus::UNAVAILABLE;
            }
            if node.is_null() {
                return FrameworkStatus::INVALID_ARGUMENT;
            }
            let position = match SpriteNodePosition::new(x, y) {
                Ok(position) => position,
                Err(error) => return FrameworkStatus::from_error(error),
            };
            // SAFETY: The caller keeps the unique live handle on the main thread for this call.
            let node = unsafe { &mut *node };
            match node.node.set_position(position) {
                Ok(()) => FrameworkStatus::OK,
                Err(error) => FrameworkStatus::from_error(error),
            }
        }
        #[cfg(not(target_os = "ios"))]
        {
            if node.is_null() {
                FrameworkStatus::INVALID_ARGUMENT
            } else {
                FrameworkStatus::UNSUPPORTED
            }
        }
    }))
}

/// Destroys one node handle and clears its original pointer slot.
///
/// # Safety
/// `node` must be null or the original aligned writable slot returned by create. On iOS, call on
/// the main thread and do not copy the handle, race another call, or destroy an alias. Off-main,
/// this function returns `FRAMEWORK_STATUS_UNAVAILABLE` without reading or changing the slot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn framework_ios_spritekit_node_destroy(
    node: *mut *mut FrameworkIosSpriteKitNode,
) -> FrameworkStatus {
    catch_unwind_status(AssertUnwindSafe(|| {
        #[cfg(target_os = "ios")]
        {
            let Some(_main_thread) = MainThread::current() else {
                return FrameworkStatus::UNAVAILABLE;
            };
            if node.is_null() {
                return FrameworkStatus::OK;
            }
            // SAFETY: The caller supplies the original writable slot returned by create.
            let value = unsafe { node.read() };
            if value.is_null() {
                return FrameworkStatus::OK;
            }
            // SAFETY: Clear the original slot before dropping its unique allocation.
            unsafe { node.write(ptr::null_mut()) };
            // SAFETY: Create returned this Box and the caller has not copied its handle.
            unsafe { drop(Box::from_raw(value)) };
            FrameworkStatus::OK
        }
        #[cfg(not(target_os = "ios"))]
        {
            let _ = node;
            FrameworkStatus::UNSUPPORTED
        }
    }))
}
