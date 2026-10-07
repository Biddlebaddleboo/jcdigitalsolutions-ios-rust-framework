#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable application lifecycle values and a static backend facade."]

use framework_core::{Availability, Result};

/// The semantic lifecycle state of an application.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum ApplicationState {
    /// The process or application has begun launch.
    Launching = 1,
    /// The application can receive foreground user input.
    Foreground = 2,
    /// The application is visible but cannot receive foreground user input.
    Inactive = 3,
    /// The application is not visible to the user.
    Background = 4,
    /// The platform has asked the application to end its work.
    Terminating = 5,
    /// The application has ended its active lifecycle.
    Terminated = 6,
}

/// A semantic application lifecycle event.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum LifecycleEvent {
    /// The backend reports a change to the current semantic state.
    StateChanged(ApplicationState),
    /// The platform has requested application termination.
    TerminationRequested,
    /// The application has ended its active lifecycle.
    Terminated,
}

/// A statically selected source of application lifecycle state and events.
///
/// Implementors own event delivery and any native callback state. This contract starts no
/// process service, event loop, thread, executor, or global registration.
pub trait ApplicationBackend {
    /// Reports whether lifecycle access is usable in the current context.
    fn availability(&self) -> Availability;

    /// Returns the current state, or a framework-owned error if the backend cannot query it.
    fn state(&self) -> Result<ApplicationState>;

    /// Returns one event already available, without blocking for a future event.
    ///
    /// Event order, queue capacity, callback thread, and overflow policy belong to the backend
    /// and must be documented by that backend.
    fn poll_event(&mut self) -> Result<Option<LifecycleEvent>>;
}

/// A thin lifecycle facade over a caller-owned, statically selected backend.
pub struct Application<B> {
    backend: B,
}

impl<B: ApplicationBackend> Application<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without performing a global lookup.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Returns the backend's current semantic lifecycle state.
    pub fn state(&self) -> Result<ApplicationState> {
        self.backend.state()
    }

    /// Reads one already available lifecycle event without waiting.
    pub fn poll_event(&mut self) -> Result<Option<LifecycleEvent>> {
        self.backend.poll_event()
    }

    /// Borrows the backend for capability-specific operations.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the backend for capability-specific operations.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Returns the backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}

/// iOS-only extension points. Native types may appear here, never in the portable facade.
pub mod ios {
    use super::ApplicationBackend;

    /// An iOS application backend that can expose its own native application handle.
    ///
    /// The associated handle type is supplied by the iOS backend; this crate does not name or
    /// wrap UIKit classes. A backend must document native ownership and thread constraints.
    pub trait IosApplicationBackend: ApplicationBackend {
        /// The native handle type chosen by this iOS backend.
        type NativeApplication;

        /// Borrows the native handle when the backend has one.
        fn native_application(&self) -> Option<&Self::NativeApplication>;
    }

    /// A borrowed view of the iOS extension implemented by an application backend.
    pub struct Extension<'a, B: IosApplicationBackend> {
        backend: &'a B,
    }

    impl<'a, B: IosApplicationBackend> Extension<'a, B> {
        /// Creates an extension view from an existing backend borrow.
        pub const fn new(backend: &'a B) -> Self {
            Self { backend }
        }

        /// Borrows the backend's native application handle when it has one.
        pub fn native_application(&self) -> Option<&B::NativeApplication> {
            self.backend.native_application()
        }
    }
}

impl<B: ios::IosApplicationBackend> Application<B> {
    /// Returns the explicit iOS-only extension view for this backend.
    pub const fn ios(&self) -> ios::Extension<'_, B> {
        ios::Extension::new(&self.backend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend {
        events: [Option<LifecycleEvent>; 2],
        next: u8,
    }

    impl ApplicationBackend for Backend {
        fn availability(&self) -> Availability {
            Availability::Available
        }

        fn state(&self) -> Result<ApplicationState> {
            Ok(ApplicationState::Foreground)
        }

        fn poll_event(&mut self) -> Result<Option<LifecycleEvent>> {
            let event = self.events.get(self.next as usize).copied().flatten();
            self.next = self.next.saturating_add(1);
            Ok(event)
        }
    }

    #[test]
    fn facade_uses_the_supplied_backend_and_nonblocking_event_poll() {
        let mut app = Application::new(Backend {
            events: [
                Some(LifecycleEvent::StateChanged(ApplicationState::Background)),
                None,
            ],
            next: 0,
        });
        assert_eq!(app.availability(), Availability::Available);
        assert_eq!(app.state(), Ok(ApplicationState::Foreground));
        assert_eq!(
            app.poll_event(),
            Ok(Some(LifecycleEvent::StateChanged(
                ApplicationState::Background
            )))
        );
        assert_eq!(app.poll_event(), Ok(None));
    }

    struct NativeApp;

    impl ios::IosApplicationBackend for Backend {
        type NativeApplication = NativeApp;

        fn native_application(&self) -> Option<&Self::NativeApplication> {
            Some(&NATIVE_APP)
        }
    }

    static NATIVE_APP: NativeApp = NativeApp;

    #[test]
    fn ios_extension_borrows_a_backend_owned_handle() {
        let app = Application::new(Backend {
            events: [None, None],
            next: 0,
        });
        assert!(app.ios().native_application().is_some());
    }
}
