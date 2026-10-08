#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable durable HTTP download values and a static backend contract."]

extern crate alloc;

use alloc::vec::Vec;
use core::num::NonZeroU128;
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};
use framework_files::AppPath;
use framework_network::{Header, HttpUrl, ResponseHeader, StatusCode};

/// A stable error for one portable download operation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum TransferError {
    /// A durable task already has this ID.
    DuplicateId,
    /// No durable task has this ID.
    NotFound,
    /// The task is not terminal and cannot be forgotten.
    NotTerminal,
    /// The backend returned a framework error.
    Backend(Error),
}

impl TransferError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::DuplicateId => ErrorKind::AlreadyExists,
            Self::NotFound => ErrorKind::NotFound,
            Self::NotTerminal => ErrorKind::InvalidInput,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::DuplicateId | Self::NotFound | Self::NotTerminal => None,
            Self::Backend(error) => error.platform_code(),
        }
    }
}

/// A nonzero, app-assigned identity for one durable download task.
///
/// Retain this exact value across app relaunch. The crate creates no ID.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct TransferId(NonZeroU128);

impl TransferId {
    /// Creates an ID from a nonzero 128-bit value.
    pub const fn new(value: u128) -> Option<Self> {
        match NonZeroU128::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Returns the exact 128-bit value.
    pub const fn get(self) -> u128 {
        self.0.get()
    }
}

/// A borrowed GET download request.
///
/// Request header order and duplicate names are kept. The backend must copy each value it needs
/// past `start_download` return.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DownloadRequest<'a> {
    id: TransferId,
    url: HttpUrl<'a>,
    headers: &'a [Header<'a>],
    destination: AppPath<'a>,
}

impl<'a> DownloadRequest<'a> {
    /// Creates a GET request from a stable ID, validated URL, headers, and destination.
    pub const fn new(
        id: TransferId,
        url: HttpUrl<'a>,
        headers: &'a [Header<'a>],
        destination: AppPath<'a>,
    ) -> Self {
        Self {
            id,
            url,
            headers,
            destination,
        }
    }

    /// Returns the app-assigned task ID.
    pub const fn id(self) -> TransferId {
        self.id
    }

    /// Returns the HTTP or HTTPS URL.
    pub const fn url(self) -> HttpUrl<'a> {
        self.url
    }

    /// Borrows request headers in app order, with duplicate names intact.
    pub const fn headers(self) -> &'a [Header<'a>] {
        self.headers
    }

    /// Returns the semantic app destination path.
    pub const fn destination(self) -> AppPath<'a> {
        self.destination
    }
}

/// Durable state for one download task.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TransferStatus {
    /// The backend accepted the task but has not reported active work.
    Queued,
    /// The backend reports active network or file work.
    Active,
    /// The response arrived and the full destination file was atomically committed.
    Succeeded {
        /// The HTTP response status; non-2xx values remain ordinary HTTP results.
        status: StatusCode,
        /// Owned response header values as observed by the backend.
        headers: Vec<ResponseHeader>,
    },
    /// The task reached a backend-reported failure.
    Failed(Error),
    /// The backend confirmed cancellation before destination commit.
    Cancelled,
}

impl TransferStatus {
    /// Reports whether this state is terminal.
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Succeeded { .. } | Self::Failed(_) | Self::Cancelled
        )
    }
}

/// A durable query result for one download task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferSnapshot {
    id: TransferId,
    status: TransferStatus,
}

impl TransferSnapshot {
    /// Creates a snapshot for one task ID and its last durable state.
    pub const fn new(id: TransferId, status: TransferStatus) -> Self {
        Self { id, status }
    }

    /// Returns the stable task ID.
    pub const fn id(&self) -> TransferId {
        self.id
    }

    /// Borrows the last durable status.
    pub const fn status(&self) -> &TransferStatus {
        &self.status
    }
}

/// A statically selected backend for durable HTTP downloads.
///
/// `start_download` must persist the task before success. A fresh backend instance must query the
/// last durable status after app relaunch. A succeeded download must publish its full destination
/// in one atomic commit; partial new bytes must never appear at the destination. Atomic commit
/// does not imply crash durability. A failed or cancelled task leaves the prior whole file or no
/// file at the destination. A backend that cannot meet these guarantees reports
/// `Availability::Unsupported`; a request-specific limit may return `ErrorKind::Unsupported`
/// before task acceptance.
pub trait TransferBackend {
    /// Reports whether durable downloads are usable in the current context.
    fn availability(&self) -> Availability;

    /// Persists one task and requests GET download work.
    ///
    /// An ID with a stored or retained record returns `TransferError::DuplicateId` without
    /// a change to that task. Success means durable acceptance, not immediate network activity. The
    /// backend must retain the request data it needs after this call.
    fn start_download(&mut self, request: DownloadRequest<'_>) -> Result<(), TransferError>;

    /// Returns the last durable task snapshot, or `None` when no record has this ID.
    fn status(&mut self, id: TransferId) -> Result<Option<TransferSnapshot>, TransferError>;

    /// Records a durable stop request for a queued or active task.
    ///
    /// A successful return means the backend recorded the request, not that task work has stopped.
    /// A status query may still report `Queued` or `Active` until a terminal result. A terminal
    /// task is a no-op success, and a terminal state may win a race. `NotFound` means no task
    /// record has this ID.
    fn cancel(&mut self, id: TransferId) -> Result<(), TransferError>;

    /// Removes a terminal task record without removing a completed destination file.
    ///
    /// A nonterminal record returns `TransferError::NotTerminal`; an absent record returns
    /// `TransferError::NotFound`. A forgotten ID may be used for a new task.
    fn forget(&mut self, id: TransferId) -> Result<(), TransferError>;
}

/// A thin download facade over an explicitly supplied backend type.
pub struct Transfers<B> {
    backend: B,
}

impl<B: TransferBackend> Transfers<B> {
    /// Creates a facade around one app-owned backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without a global lookup.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Requests one durable GET download.
    pub fn start_download(&mut self, request: DownloadRequest<'_>) -> Result<(), TransferError> {
        self.backend.start_download(request)
    }

    /// Queries a task by its stable ID.
    pub fn status(&mut self, id: TransferId) -> Result<Option<TransferSnapshot>, TransferError> {
        self.backend.status(id)
    }

    /// Requests task cancellation without removal of its durable record.
    pub fn cancel(&mut self, id: TransferId) -> Result<(), TransferError> {
        self.backend.cancel(id)
    }

    /// Forgets a terminal task record while retaining a completed destination file.
    pub fn forget(&mut self, id: TransferId) -> Result<(), TransferError> {
        self.backend.forget(id)
    }

    /// Borrows the backend for operations outside this contract.
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutably borrows the backend for operations outside this contract.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Returns the backend and ends this facade borrow.
    pub fn into_backend(self) -> B {
        self.backend
    }
}
