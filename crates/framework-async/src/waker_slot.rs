use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Waker;

use crate::operation::CancellationRegistration;

struct WakerData {
    next_id: u32,
    current: Option<(u32, Waker)>,
}

pub(crate) struct WakerSlot {
    locked: AtomicBool,
    data: UnsafeCell<WakerData>,
}

// Access to WakerData is serialized by `locked`; Waker is Send + Sync.
unsafe impl Sync for WakerSlot {}

struct Lock<'a> {
    slot: &'a WakerSlot,
}

impl core::ops::Deref for Lock<'_> {
    type Target = WakerData;

    fn deref(&self) -> &Self::Target {
        // SAFETY: the lock guard is created only after an Acquire transition to `locked`.
        unsafe { &*self.slot.data.get() }
    }
}

impl core::ops::DerefMut for Lock<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: this guard owns the exclusive lock for the lifetime of the mutable borrow.
        unsafe { &mut *self.slot.data.get() }
    }
}

impl Drop for Lock<'_> {
    fn drop(&mut self) {
        self.slot.locked.store(false, Ordering::Release);
    }
}

impl WakerSlot {
    pub(crate) const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(WakerData {
                next_id: 0,
                current: None,
            }),
        }
    }

    fn lock(&self) -> Lock<'_> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                spin_loop();
            }
        }
        Lock { slot: self }
    }

    pub(crate) fn register(&self, waker: &Waker) -> CancellationRegistration<'_> {
        let clone = waker.clone();
        let mut data = self.lock();
        if let Some((id, current)) = &data.current {
            if current.will_wake(waker) {
                let id = *id;
                drop(data);
                drop(clone);
                return CancellationRegistration { waker: self, id };
            }
        }
        data.next_id = data.next_id.wrapping_add(1);
        if data.next_id == 0 {
            data.next_id = 1;
        }
        let id = data.next_id;
        let previous = data.current.replace((id, clone));
        drop(data);
        drop(previous);
        CancellationRegistration { waker: self, id }
    }

    pub(crate) fn unregister(&self, id: u32) {
        let mut data = self.lock();
        let previous = match data.current.as_ref() {
            Some((current_id, _)) if *current_id == id => data.current.take(),
            _ => None,
        };
        drop(data);
        drop(previous);
    }

    pub(crate) fn wake(&self) {
        let mut data = self.lock();
        let current = data.current.take();
        drop(data);
        if let Some((_, waker)) = current {
            waker.wake();
        }
    }
}
