#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Narrow File Provider registered-domain snapshots for iOS"]

mod completion;

#[cfg(target_os = "ios")]
mod platform;

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::sync::Arc;

use completion::Completion;

/// One point-in-time result for the calling app's own File Provider extension
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegisteredDomainPresence {
    has_registered_domains: bool,
}

impl RegisteredDomainPresence {
    /// Creates a snapshot value
    pub const fn new(has_registered_domains: bool) -> Self {
        Self {
            has_registered_domains,
        }
    }

    /// Returns whether the calling app's own provider has one or more registered domains
    pub const fn has_registered_domains(self) -> bool {
        self.has_registered_domains
    }
}

/// A fixed-width count snapshot for the calling app's own File Provider domains
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegisteredDomainCount {
    count: u64,
}

impl RegisteredDomainCount {
    /// Creates a count snapshot
    pub const fn new(count: u64) -> Self {
        Self { count }
    }

    /// Returns the number of registered domains at query time
    pub const fn count(self) -> u64 {
        self.count
    }
}

/// An owned FileProvider `NSError` domain and native `NSInteger` code
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeFileProviderError {
    domain: String,
    code: isize,
}

impl NativeFileProviderError {
    /// Creates an owned native error value
    pub fn new(domain: String, code: isize) -> Self {
        Self { domain, code }
    }

    /// Returns the native error domain
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Returns the native `NSInteger` error code
    pub const fn code(&self) -> isize {
        self.code
    }
}

/// A File Provider query error
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileProviderQueryError {
    /// The API is unavailable on this target
    UnsupportedPlatform,
    /// The API is unavailable at the current iOS version
    ApiUnavailable,
    /// The native API returned an error; its domain and code are preserved
    Native(NativeFileProviderError),
    /// The native domain count did not fit the fixed-width result
    CountOutOfRange,
    /// The native callback encountered a Rust panic
    CallbackPanicked,
}

/// A one-shot, caller-owned future for a registered-domain presence snapshot
#[must_use = "the future carries the requested File Provider query result"]
pub struct RegisteredDomainPresenceFuture {
    completion: Arc<Completion>,
    finished: bool,
}

impl RegisteredDomainPresenceFuture {
    fn new(completion: Arc<Completion>) -> Self {
        Self {
            completion,
            finished: false,
        }
    }
}

impl Future for RegisteredDomainPresenceFuture {
    type Output = Result<RegisteredDomainPresence, FileProviderQueryError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.finished {
            return Poll::Pending;
        }
        match this.completion.poll(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result.map(|count| RegisteredDomainPresence::new(count != 0)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A one-shot, caller-owned future for a registered-domain count snapshot
#[must_use = "the future carries the requested File Provider query result"]
pub struct RegisteredDomainCountFuture {
    completion: Arc<Completion>,
    finished: bool,
}

impl RegisteredDomainCountFuture {
    fn new(completion: Arc<Completion>) -> Self {
        Self {
            completion,
            finished: false,
        }
    }
}

impl Future for RegisteredDomainCountFuture {
    type Output = Result<RegisteredDomainCount, FileProviderQueryError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.finished {
            return Poll::Pending;
        }
        match this.completion.poll(context) {
            Poll::Ready(result) => {
                this.finished = true;
                Poll::Ready(result.map(RegisteredDomainCount::new))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for RegisteredDomainCountFuture {
    fn drop(&mut self) {
        self.completion.detach();
    }
}

impl Drop for RegisteredDomainPresenceFuture {
    fn drop(&mut self) {
        self.completion.detach();
    }
}

/// Starts a query for registered domains belonging only to this app's own File Provider extension
///
/// The native request starts when this function runs, not when the returned future is first polled
/// Dropping the future abandons Rust interest but cannot cancel the native request
/// The result does not report provider enablement, online state, sync, file access, or other providers
pub fn request_registered_domain_presence() -> RegisteredDomainPresenceFuture {
    let completion = Arc::new(Completion::new());

    #[cfg(target_os = "ios")]
    platform::start(Arc::clone(&completion));

    #[cfg(not(target_os = "ios"))]
    completion.complete(Err(FileProviderQueryError::UnsupportedPlatform));

    RegisteredDomainPresenceFuture::new(completion)
}

/// Starts a count query for domains belonging only to this app's own File Provider extension
///
/// The native request starts when this function runs, not when the returned future is first polled
/// Each call starts a separate query; it exposes no domain identifiers or other domain metadata
pub fn request_registered_domain_count() -> RegisteredDomainCountFuture {
    let completion = Arc::new(Completion::new());

    #[cfg(target_os = "ios")]
    platform::start(Arc::clone(&completion));

    #[cfg(not(target_os = "ios"))]
    completion.complete(Err(FileProviderQueryError::UnsupportedPlatform));

    RegisteredDomainCountFuture::new(completion)
}
