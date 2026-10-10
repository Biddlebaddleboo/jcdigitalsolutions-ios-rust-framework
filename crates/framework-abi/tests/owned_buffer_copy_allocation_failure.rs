use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};
use framework_abi::{
    framework_owned_buffer_copy, FrameworkOwnedBuffer, FrameworkSlice, FrameworkStatus,
};
use std::alloc::System;

struct OneShotFailingAllocator;

static FAIL_LAYOUT_SIZE: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static TEST_ALLOCATOR: OneShotFailingAllocator = OneShotFailingAllocator;

fn fail_sized_allocation(size: usize) -> bool {
    FAIL_LAYOUT_SIZE
        .compare_exchange(size, 0, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
}

// SAFETY: all allocation operations delegate to `System`, except for one explicitly armed
// allocation size that returns null to exercise the ABI's fallible-reservation path.
unsafe impl GlobalAlloc for OneShotFailingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if fail_sized_allocation(layout.size()) {
            ptr::null_mut()
        } else {
            // SAFETY: `layout` is the request provided to this global allocator.
            unsafe { System.alloc(layout) }
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if fail_sized_allocation(layout.size()) {
            ptr::null_mut()
        } else {
            // SAFETY: `layout` is the request provided to this global allocator.
            unsafe { System.alloc_zeroed(layout) }
        }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: pointers returned by this allocator came from `System`.
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if fail_sized_allocation(new_size) {
            ptr::null_mut()
        } else {
            // SAFETY: `pointer` and `layout` are the live allocation supplied to this allocator.
            unsafe { System.realloc(pointer, layout, new_size) }
        }
    }
}

#[test]
fn owned_buffer_copy_reports_reservation_failure() {
    const FAIL_SIZE: usize = 4093;
    let input_bytes = [0xa5; FAIL_SIZE];
    let input = FrameworkSlice::from_bytes(&input_bytes).unwrap();
    let mut output = FrameworkOwnedBuffer::default();

    FAIL_LAYOUT_SIZE.store(FAIL_SIZE, Ordering::SeqCst);
    // SAFETY: `output` is aligned writable empty storage and `input_bytes` is live/readable.
    let status = unsafe { framework_owned_buffer_copy(input, &mut output) };
    let unconsumed_failure = FAIL_LAYOUT_SIZE.swap(0, Ordering::SeqCst);

    assert_eq!(
        unconsumed_failure, 0,
        "the reserved allocation was not attempted"
    );
    assert_eq!(status, FrameworkStatus::RESOURCE_EXHAUSTED);
    assert!(output.data().is_null());
    assert_eq!(output.length(), 0);
    assert_eq!(output.capacity(), 0);
}
