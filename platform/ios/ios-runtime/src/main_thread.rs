//! Typed proof that the current call runs on UIKit's main thread.

use objc2::MainThreadMarker;

/// Main-thread proof for APIs that require UIKit access.
///
/// This adapter preserves `objc2`'s non-sendable marker. It has no runtime lookup beyond the
/// marker check and is specific to the iOS backend, not the portable framework API.
pub struct MainThread(MainThreadMarker);

impl MainThread {
    /// Return a proof when called on the main thread.
    pub fn current() -> Option<Self> {
        MainThreadMarker::new().map(Self)
    }

    /// Consume this proof to call `objc2` APIs that require their native marker.
    pub fn into_objc2(self) -> MainThreadMarker {
        self.0
    }
}
