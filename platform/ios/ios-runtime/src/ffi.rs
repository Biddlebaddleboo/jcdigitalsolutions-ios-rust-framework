//! Helpers for Rust callbacks reached from C or Objective-C.

/// Marker returned when a Rust callback panics before the panic can cross a foreign ABI frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PanicCaught;

/// Run a callback without allowing a Rust unwind to cross an Objective-C or C boundary.
///
/// The panic hook still runs. With `panic=abort`, the process aborts before this helper can
/// recover; neither strategy permits an unwind into Apple code. Callers must choose a safe
/// native fallback when this returns `Err`.
pub fn catch_unwind<R>(callback: impl FnOnce() -> R) -> Result<R, PanicCaught> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback)).map_err(|_| PanicCaught)
}
