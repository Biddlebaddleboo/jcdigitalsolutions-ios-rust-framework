#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable, bounded caller-selected Bonjour discovery contracts with static backend selection."]

extern crate alloc;

use alloc::string::String;
use core::future::Future;

/// Maximum UTF-8 byte length for a Bonjour type, domain, or returned service identity component.
pub const MAX_BONJOUR_COMPONENT_BYTES: usize = 255;

/// Maximum number of queued service add/remove events a backend may retain.
pub const MAX_QUEUED_BONJOUR_EVENTS: usize = 32;

/// A caller-selected Bonjour service type and optional browsing domain.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BonjourQuery {
    service_type: String,
    domain: Option<String>,
}

impl BonjourQuery {
    /// Creates a query for one specific `_name._tcp` or `_name._udp` service type.
    ///
    /// The optional domain must be an ASCII DNS name; `None` asks the backend to browse its
    /// default domains. Input is preserved exactly and is not canonicalized.
    pub fn new(service_type: &str, domain: Option<&str>) -> Result<Self, BonjourBrowseError> {
        if !valid_service_type(service_type) {
            return Err(BonjourBrowseError::InvalidServiceType);
        }
        let service_type = copy_bounded(service_type)?;
        let domain = match domain {
            Some(value) if valid_domain(value) => Some(copy_bounded(value)?),
            Some(_) => return Err(BonjourBrowseError::InvalidDomain),
            None => None,
        };
        Ok(Self {
            service_type,
            domain,
        })
    }

    /// Returns the exact selected Bonjour service type.
    pub fn service_type(&self) -> &str {
        &self.service_type
    }

    /// Returns the exact selected domain, or `None` for the backend's default browse domains.
    pub fn domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }
}

/// The exact Bonjour endpoint identity reported for one discovered service.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BonjourService {
    name: String,
    service_type: String,
    domain: String,
}

impl BonjourService {
    /// Creates an owned identity while preserving each supplied UTF-8 value without normalization.
    pub fn new(name: &str, service_type: &str, domain: &str) -> Result<Self, BonjourBrowseError> {
        if name.is_empty() || !valid_component(name) {
            return Err(BonjourBrowseError::InvalidNativeService);
        }
        if !valid_service_type(service_type) {
            return Err(BonjourBrowseError::InvalidNativeService);
        }
        if !valid_domain(domain) {
            return Err(BonjourBrowseError::InvalidNativeService);
        }
        Ok(Self {
            name: copy_bounded(name)?,
            service_type: copy_bounded(service_type)?,
            domain: copy_bounded(domain)?,
        })
    }

    /// Creates an owned identity from exact, bounded UTF-8 byte components.
    ///
    /// This lets platform backends reject malformed native strings without lossy conversion.
    pub fn from_utf8_components(
        name: &[u8],
        service_type: &[u8],
        domain: &[u8],
    ) -> Result<Self, BonjourBrowseError> {
        let name =
            core::str::from_utf8(name).map_err(|_| BonjourBrowseError::InvalidNativeService)?;
        let service_type = core::str::from_utf8(service_type)
            .map_err(|_| BonjourBrowseError::InvalidNativeService)?;
        let domain =
            core::str::from_utf8(domain).map_err(|_| BonjourBrowseError::InvalidNativeService)?;
        Self::new(name, service_type, domain)
    }

    /// Returns the exact UTF-8 service instance name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the exact Bonjour service type.
    pub fn service_type(&self) -> &str {
        &self.service_type
    }

    /// Returns the exact resolved Bonjour domain.
    pub fn domain(&self) -> &str {
        &self.domain
    }
}

/// A raw native error domain and code from the selected browser backend.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NativeBonjourError {
    domain: i32,
    code: i32,
}

impl NativeBonjourError {
    /// Creates an error without translating either backend value.
    pub const fn new(domain: i32, code: i32) -> Self {
        Self { domain, code }
    }

    /// Returns the raw native error domain.
    pub const fn domain(self) -> i32 {
        self.domain
    }

    /// Returns the raw native error code.
    pub const fn code(self) -> i32 {
        self.code
    }
}

/// A state snapshot reported by a Bonjour browser backend.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum BonjourBrowserState {
    /// The browser has not received its first native state update.
    Starting,
    /// The browser is ready to deliver service changes.
    Ready,
    /// The browser is waiting for connectivity; browsing may resume later.
    Waiting(Option<NativeBonjourError>),
    /// The native browser failed and cannot resume.
    Failed(Option<NativeBonjourError>),
    /// The browser reached its final cancellation callback.
    Closed,
}

/// One service-membership change or coalesced browser-state update.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BonjourBrowserEvent {
    /// A caller-selected Bonjour service became available.
    ServiceAdded(BonjourService),
    /// A previously available Bonjour service is no longer present.
    ServiceRemoved(BonjourService),
    /// The latest browser state; intermediate state updates may be coalesced.
    State(BonjourBrowserState),
}

/// A query, delivery, lifecycle, or native browsing failure.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum BonjourBrowseError {
    /// The selected service type is not a bounded ASCII Bonjour service type.
    InvalidServiceType,
    /// The selected domain is not a bounded ASCII DNS domain.
    InvalidDomain,
    /// The backend operation is invalid in its current lifecycle state.
    InvalidState,
    /// An owned query, identity, or event queue allocation failed.
    ResourceExhausted,
    /// The bounded event queue filled; the backend stopped to avoid silent partial discovery.
    QueueOverflow,
    /// Native service identity data was null, over-bound, or invalid UTF-8.
    InvalidNativeService,
    /// The browser was cancelled or closed.
    Cancelled,
    /// The backend failed without a native error value.
    BackendFailure,
    /// The backend returned a raw native error domain and code.
    Native(NativeBonjourError),
}

/// A statically selected backend for one caller-owned Bonjour browser.
///
/// `start` is lazy: it must not allocate or start native work before the returned future is first
/// polled. `next_event` may remain pending until a service or state change arrives; it has no
/// timeout or executor requirement. Backends must bound queued service changes to
/// [`MAX_QUEUED_BONJOUR_EVENTS`], coalesce state to its latest value, and return
/// [`BonjourBrowseError::QueueOverflow`] rather than silently discard membership changes. A
/// dropped `next_event` future detaches only its waiter. `close` waits for the native cancellation
/// boundary; `cancel` and drop request nonblocking cancellation. Callback storage must remain alive
/// through the final native cancellation callback. The backend invokes no user callback on a native
/// queue, but it may wake a stored waker there after releasing its locks.
pub trait BonjourBrowserBackend {
    /// The concrete lazy start future selected without trait-object boxing.
    type StartFuture<'a>: Future<Output = Result<(), BonjourBrowseError>> + 'a
    where
        Self: 'a;

    /// The concrete future for one service or browser-state event.
    type NextEventFuture<'a>: Future<Output = Result<BonjourBrowserEvent, BonjourBrowseError>> + 'a
    where
        Self: 'a;

    /// The concrete future for native browser cancellation completion.
    type CloseFuture<'a>: Future<Output = Result<(), BonjourBrowseError>> + 'a
    where
        Self: 'a;

    /// Starts browsing for the selected service type and optional domain when first polled.
    fn start<'a>(&'a mut self, query: BonjourQuery) -> Self::StartFuture<'a>;

    /// Waits for the next bounded membership event or latest browser-state update.
    fn next_event<'a>(&'a mut self) -> Self::NextEventFuture<'a>;

    /// Requests cancellation and resolves after native callback cleanup completes.
    fn close<'a>(&'a mut self) -> Self::CloseFuture<'a>;

    /// Requests idempotent, nonblocking cancellation.
    fn cancel(&mut self);
}

/// A thin facade over caller-owned, statically selected Bonjour-browser state.
pub struct BonjourBrowser<B: BonjourBrowserBackend> {
    backend: B,
}

impl<B: BonjourBrowserBackend> BonjourBrowser<B> {
    /// Creates a browser facade without starting native work.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Starts browsing for one explicitly selected Bonjour query.
    pub fn start(&mut self, query: BonjourQuery) -> B::StartFuture<'_> {
        self.backend.start(query)
    }

    /// Waits for the next service-membership or latest state event.
    pub fn next_event(&mut self) -> B::NextEventFuture<'_> {
        self.backend.next_event()
    }

    /// Requests cancellation and waits for native cleanup.
    pub fn close(&mut self) -> B::CloseFuture<'_> {
        self.backend.close()
    }

    /// Requests nonblocking cancellation.
    pub fn cancel(&mut self) {
        self.backend.cancel();
    }
}

impl<B: BonjourBrowserBackend> Drop for BonjourBrowser<B> {
    fn drop(&mut self) {
        self.backend.cancel();
    }
}

fn copy_bounded(value: &str) -> Result<String, BonjourBrowseError> {
    if !valid_component(value) {
        return Err(BonjourBrowseError::InvalidNativeService);
    }
    let mut copy = String::new();
    copy.try_reserve_exact(value.len())
        .map_err(|_| BonjourBrowseError::ResourceExhausted)?;
    copy.push_str(value);
    Ok(copy)
}

fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_BONJOUR_COMPONENT_BYTES
        && !value.as_bytes().contains(&0)
}

fn valid_service_type(value: &str) -> bool {
    if !valid_component(value) || !value.is_ascii() {
        return false;
    }
    let mut labels = value.split('.');
    let service = labels.next().unwrap_or_default();
    let transport = labels.next().unwrap_or_default();
    labels.next().is_none() && service_label(service) && matches!(transport, "_tcp" | "_udp")
}

fn service_label(label: &str) -> bool {
    if label.len() < 2 || label.len() > 16 || !label.starts_with('_') {
        return false;
    }
    let bytes = &label.as_bytes()[1..];
    bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
}

fn valid_domain(value: &str) -> bool {
    if !valid_component(value) || !value.is_ascii() {
        return false;
    }
    let value = value.strip_suffix('.').unwrap_or(value);
    if value.is_empty() {
        return false;
    }
    value.split('.').all(|label| {
        let bytes = label.as_bytes();
        !bytes.is_empty()
            && bytes.len() <= 63
            && bytes.first().is_some_and(u8::is_ascii_alphanumeric)
            && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
    })
}
