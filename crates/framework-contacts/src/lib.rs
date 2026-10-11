#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable Contacts authorization values and a runtime-neutral static backend contract."]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::future::Future;
use framework_core::{Availability, Error, ErrorKind, PlatformErrorCode};

/// The normalized authorization state for access to contacts.
///
/// `Limited` is intentionally distinct from [`Authorized`](Self::Authorized): it permits access
/// to only the contacts selected by the user and does not mean full contact-store access.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum ContactsAuthorization {
    /// The native state is not recognized or cannot be classified.
    Unknown = 0,
    /// The user has not yet made a choice.
    NotDetermined = 1,
    /// Access is blocked by policy or device restrictions.
    Restricted = 2,
    /// The user denied contact access.
    Denied = 3,
    /// Full access to contacts is authorized.
    Authorized = 4,
    /// Access is authorized only for a user-selected subset of contacts.
    Limited = 5,
}

impl ContactsAuthorization {
    /// Reports whether the state permits access to any contact data.
    pub const fn allows_contact_access(self) -> bool {
        matches!(self, Self::Authorized | Self::Limited)
    }

    /// Reports whether the state represents full, rather than user-limited, access.
    pub const fn is_full_access(self) -> bool {
        matches!(self, Self::Authorized)
    }
}

/// A stable Contacts operation error with an optional backend-native code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ContactsError {
    /// The supplied local contact identifier is empty or contains a NUL character.
    InvalidIdentifier,
    /// The supplied name search query is empty or contains a NUL character.
    InvalidSearchQuery,
    /// The supplied contact draft has no useful data or contains an empty phone/email value.
    InvalidContactDraft,
    /// The supplied contact update changes no fields or contains an empty phone/email value.
    InvalidContactUpdate,
    /// The current authorization state does not permit this Contacts operation.
    AuthorizationRequired(ContactsAuthorization),
    /// No accessible contact matches the supplied local identifier.
    NotFound,
    /// The selected backend returned an operation error rather than an authorization result.
    Backend(Error),
}

impl ContactsError {
    /// Returns the stable portable error category.
    pub const fn kind(self) -> ErrorKind {
        match self {
            Self::InvalidIdentifier
            | Self::InvalidSearchQuery
            | Self::InvalidContactDraft
            | Self::InvalidContactUpdate => ErrorKind::InvalidInput,
            Self::AuthorizationRequired(
                ContactsAuthorization::Denied | ContactsAuthorization::Restricted,
            ) => ErrorKind::PermissionDenied,
            Self::AuthorizationRequired(_) => ErrorKind::Unavailable,
            Self::NotFound => ErrorKind::NotFound,
            Self::Backend(error) => error.kind(),
        }
    }

    /// Returns the optional backend-native error code.
    pub const fn platform_code(self) -> Option<PlatformErrorCode> {
        match self {
            Self::Backend(error) => error.platform_code(),
            Self::InvalidIdentifier
            | Self::InvalidSearchQuery
            | Self::InvalidContactDraft
            | Self::InvalidContactUpdate
            | Self::AuthorizationRequired(_)
            | Self::NotFound => None,
        }
    }
}

/// An owned snapshot of common fields from one Contacts record.
///
/// Phone and email values retain the native order returned by Contacts. That order is not a
/// stable sort guarantee. A local identifier can become stale when the Contacts database changes
/// or records are unified.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contact {
    /// The local Contacts identifier for this record.
    pub identifier: String,
    /// The given name, or an empty string when absent.
    pub given_name: String,
    /// The family name, or an empty string when absent.
    pub family_name: String,
    /// The organization name, or an empty string when absent.
    pub organization_name: String,
    /// The record's labeled phone numbers in native order.
    pub phone_numbers: Vec<ContactLabeledValue>,
    /// The record's labeled email addresses in native order.
    pub email_addresses: Vec<ContactLabeledValue>,
}

/// An owned label/value pair for one phone number or email address.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactLabeledValue {
    /// The native label, when present. Labels may be localized by the system.
    pub label: Option<String>,
    /// The phone number or email address value.
    pub value: String,
}

/// Rust-owned fields for one new contact record.
///
/// At least one name, organization, phone, or email value must be non-empty. Phone/email values
/// may use an optional native label; values are passed to the selected backend unchanged.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ContactDraft {
    /// The new contact's given name, or empty when not supplied.
    pub given_name: String,
    /// The new contact's family name, or empty when not supplied.
    pub family_name: String,
    /// The new contact's organization, or empty when not supplied.
    pub organization_name: String,
    /// Labeled phone values in caller order.
    pub phone_numbers: Vec<ContactLabeledValue>,
    /// Labeled email values in caller order.
    pub email_addresses: Vec<ContactLabeledValue>,
}

/// Rust-owned changes for selected fields of one existing contact.
///
/// `None` preserves that property. An empty string clears a selected string property, and an
/// empty vector clears the selected phone or email property. A nonempty phone/email vector
/// replaces that property's full value list in caller order.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ContactUpdate {
    /// The replacement given name, or `None` to preserve it.
    pub given_name: Option<String>,
    /// The replacement family name, or `None` to preserve it.
    pub family_name: Option<String>,
    /// The replacement organization name, or `None` to preserve it.
    pub organization_name: Option<String>,
    /// Replacement labeled phone numbers, or `None` to preserve them.
    pub phone_numbers: Option<Vec<ContactLabeledValue>>,
    /// Replacement labeled email addresses, or `None` to preserve them.
    pub email_addresses: Option<Vec<ContactLabeledValue>>,
}

/// A statically selected backend for Contacts authorization status and request operations.
///
/// [`authorization_status`](Self::authorization_status) must not show a permission prompt. A
/// request may prompt only after its returned future is first polled. Dropping an unpolled future
/// must have no prompt side effect. Dropping a started future abandons interest in its result but
/// cannot be assumed to dismiss a system prompt already shown. Callback state must not borrow
/// future-owned storage and must remain valid until the native operation reaches one terminal
/// outcome. The first terminal completion becomes one future result; duplicate native completions
/// must not produce another result. A live future yields its result at most once. No global
/// service, dynamic dispatch, or executor is required by this contract.
pub trait ContactsBackend {
    /// Reports whether the backend's authorization facility is available in this context.
    fn availability(&self) -> Availability;

    /// Queries authorization without prompting or starting an access request.
    fn authorization_status(&self) -> ContactsAuthorization;

    /// The future type for an explicit permission request.
    type RequestAuthorizationFuture<'a>: Future<Output = Result<ContactsAuthorization, ContactsError>>
        + 'a
    where
        Self: 'a;

    /// Explicitly requests contact authorization.
    ///
    /// The backend must start prompt-capable native work only when this future is first polled.
    /// A completed denial or limited grant is an authorization result, not inherently an
    /// operation error. Dropping the future cannot be assumed to cancel a prompt already shown;
    /// callback state must remain valid until native completion without borrowing future-owned
    /// storage. Duplicate native completions must not produce another future result.
    fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a>;
}

/// A read-only extension for backends that can fetch one record by local identifier.
///
/// Implementations must re-check authorization for each call, permit only `Authorized` or
/// `Limited`, reject empty/NUL-containing identifiers, and return owned Rust data. This separate
/// trait keeps existing [`ContactsBackend`] implementors source-compatible.
pub trait ContactsFetchBackend: ContactsBackend {
    /// Fetches one accessible contact by caller-supplied local identifier.
    fn fetch_contact(&self, identifier: &str) -> Result<Contact, ContactsError>;
}

/// A read-only extension for backends that search contacts by the platform's name predicate.
///
/// Implementations must re-check authorization for each valid query, permit only `Authorized` or
/// `Limited`, reject empty/NUL queries, and return no more than `max_results` owned snapshots.
/// A zero limit returns no results without native enumeration, after the authorization check.
/// Query matching and result order follow the selected native backend and are not otherwise
/// normalized by this contract. This separate trait keeps existing [`ContactsBackend`]
/// implementors source-compatible.
pub trait ContactsSearchBackend: ContactsBackend {
    /// Searches contacts by name without prompting or mutating the store.
    fn search_contacts_by_name(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<Vec<Contact>, ContactsError>;
}

/// An add-only extension for backends that can create one new contact record.
///
/// Implementations must validate the draft, re-check authorization without prompting, permit
/// only `Authorized` or `Limited`, and add one new record without updating or deleting existing
/// records. This separate trait keeps existing [`ContactsBackend`] implementors source-compatible.
pub trait ContactsCreateBackend: ContactsBackend {
    /// Adds one new record from owned Rust values and returns no platform identifier.
    fn create_contact(&self, draft: &ContactDraft) -> Result<(), ContactsError>;
}

/// An update extension for backends that can modify one existing contact by local identifier.
///
/// Implementations must reject invalid identifiers and no-op patches, re-check authorization
/// without prompting, permit only `Authorized` or `Limited`, and update no fields absent from the
/// patch. A `Some` empty string or vector clears that selected property. This separate trait
/// keeps existing [`ContactsBackend`] implementors source-compatible.
pub trait ContactsUpdateBackend: ContactsBackend {
    /// Updates selected common fields on one accessible contact without prompting.
    fn update_contact(&self, identifier: &str, update: &ContactUpdate)
    -> Result<(), ContactsError>;
}

/// A delete extension for backends that can remove one existing contact by local identifier.
///
/// Implementations must reject invalid identifiers, re-check authorization without prompting,
/// permit only `Authorized` or `Limited`, and delete only the one caller-selected accessible
/// record. This separate trait keeps existing [`ContactsBackend`] implementors source-compatible.
pub trait ContactsDeleteBackend: ContactsBackend {
    /// Deletes one accessible contact without prompting, enumerating, or changing groups.
    fn delete_contact(&self, identifier: &str) -> Result<(), ContactsError>;
}

/// A read-only extension for backends that export one record as native vCard bytes.
///
/// Implementations must reject empty/NUL identifiers, re-check authorization without prompting,
/// permit only `Authorized` or `Limited`, fetch only the supplied identifier with the native
/// serializer's required keys, and return owned bytes without text conversion. This separate
/// trait keeps existing [`ContactsBackend`] implementors source-compatible.
pub trait ContactsExportBackend: ContactsBackend {
    /// Exports one accessible contact as an owned vCard byte sequence without prompting.
    fn export_contact_vcard(&self, identifier: &str) -> Result<Vec<u8>, ContactsError>;
}

/// A thin facade over caller-owned, statically selected Contacts backend state.
pub struct Contacts<B> {
    backend: B,
}

impl<B: ContactsBackend> Contacts<B> {
    /// Creates a facade around an explicitly supplied backend.
    pub const fn new(backend: B) -> Self {
        Self { backend }
    }

    /// Reports backend availability without global lookup or hidden initialization.
    pub fn availability(&self) -> Availability {
        self.backend.availability()
    }

    /// Queries authorization without prompting.
    pub fn authorization_status(&self) -> ContactsAuthorization {
        self.backend.authorization_status()
    }

    /// Starts an explicit authorization request when the returned future is first polled.
    pub fn request_authorization(&mut self) -> B::RequestAuthorizationFuture<'_> {
        self.backend.request_authorization()
    }
}

impl<B: ContactsFetchBackend> Contacts<B> {
    /// Fetches one contact without prompting, enumerating, or mutating the contact store.
    pub fn fetch_contact(&self, identifier: &str) -> Result<Contact, ContactsError> {
        self.backend.fetch_contact(identifier)
    }
}

impl<B: ContactsSearchBackend> Contacts<B> {
    /// Searches contacts by the backend's native name-matching semantics, without prompting.
    pub fn search_contacts_by_name(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<Vec<Contact>, ContactsError> {
        self.backend.search_contacts_by_name(query, max_results)
    }
}

impl<B: ContactsCreateBackend> Contacts<B> {
    /// Adds one new contact without prompting or changing an existing record.
    pub fn create_contact(&self, draft: &ContactDraft) -> Result<(), ContactsError> {
        self.backend.create_contact(draft)
    }
}

impl<B: ContactsUpdateBackend> Contacts<B> {
    /// Updates selected fields of one contact without prompting or changing other fields.
    pub fn update_contact(
        &self,
        identifier: &str,
        update: &ContactUpdate,
    ) -> Result<(), ContactsError> {
        self.backend.update_contact(identifier, update)
    }
}

impl<B: ContactsDeleteBackend> Contacts<B> {
    /// Deletes one contact without prompting or enumerating the contact store.
    pub fn delete_contact(&self, identifier: &str) -> Result<(), ContactsError> {
        self.backend.delete_contact(identifier)
    }
}

impl<B: ContactsExportBackend> Contacts<B> {
    /// Exports one contact as native vCard bytes without prompting or enumerating contacts.
    pub fn export_contact_vcard(&self, identifier: &str) -> Result<Vec<u8>, ContactsError> {
        self.backend.export_contact_vcard(identifier)
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use core::pin::Pin;
    use core::task::{Context, Poll, Waker};
    use std::cell::Cell;
    use std::rc::Rc;
    use std::vec;

    struct FakeBackend {
        status: ContactsAuthorization,
        requests: Rc<Cell<usize>>,
    }

    struct FakeRequestFuture<'a> {
        backend: &'a mut FakeBackend,
        completed: bool,
    }

    impl Future for FakeRequestFuture<'_> {
        type Output = Result<ContactsAuthorization, ContactsError>;

        fn poll(mut self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
            if self.completed {
                return Poll::Pending;
            }
            let backend = &mut self.backend;
            backend.requests.set(backend.requests.get() + 1);
            let status = backend.status;
            self.completed = true;
            Poll::Ready(Ok(status))
        }
    }

    impl ContactsBackend for FakeBackend {
        fn availability(&self) -> Availability {
            Availability::Available
        }

        fn authorization_status(&self) -> ContactsAuthorization {
            self.status
        }

        type RequestAuthorizationFuture<'a> = FakeRequestFuture<'a>;

        fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a> {
            FakeRequestFuture {
                backend: self,
                completed: false,
            }
        }
    }

    impl ContactsCreateBackend for FakeBackend {
        fn create_contact(&self, draft: &ContactDraft) -> Result<(), ContactsError> {
            if draft.given_name.is_empty() {
                Err(ContactsError::InvalidContactDraft)
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn authorization_values_keep_limited_separate_from_full_access() {
        assert_eq!(core::mem::size_of::<ContactsAuthorization>(), 1);
        assert_eq!(ContactsAuthorization::Unknown as u8, 0);
        assert_eq!(ContactsAuthorization::NotDetermined as u8, 1);
        assert_eq!(ContactsAuthorization::Restricted as u8, 2);
        assert_eq!(ContactsAuthorization::Denied as u8, 3);
        assert_eq!(ContactsAuthorization::Authorized as u8, 4);
        assert_eq!(ContactsAuthorization::Limited as u8, 5);
        assert!(!ContactsAuthorization::Unknown.allows_contact_access());
        assert!(!ContactsAuthorization::NotDetermined.allows_contact_access());
        assert!(ContactsAuthorization::Authorized.allows_contact_access());
        assert!(ContactsAuthorization::Authorized.is_full_access());
        assert!(ContactsAuthorization::Limited.allows_contact_access());
        assert!(!ContactsAuthorization::Limited.is_full_access());
        assert!(!ContactsAuthorization::Denied.allows_contact_access());
        assert!(!ContactsAuthorization::Restricted.allows_contact_access());
    }

    #[test]
    fn query_does_not_prompt_and_request_is_lazy() {
        let backend = FakeBackend {
            status: ContactsAuthorization::Limited,
            requests: Rc::new(Cell::new(0)),
        };
        let requests = backend.requests.clone();
        let mut contacts = Contacts::new(backend);
        assert_eq!(contacts.availability(), Availability::Available);
        assert_eq!(
            contacts.authorization_status(),
            ContactsAuthorization::Limited
        );
        assert_eq!(requests.get(), 0);

        {
            let _future = contacts.request_authorization();
            assert_eq!(requests.get(), 0);
        }
        assert_eq!(requests.get(), 0);

        assert_eq!(requests.get(), 0);
        {
            let mut future = contacts.request_authorization();
            let mut context = Context::from_waker(Waker::noop());
            assert!(matches!(
                Pin::new(&mut future).poll(&mut context),
                Poll::Ready(Ok(ContactsAuthorization::Limited))
            ));
        }
        assert_eq!(requests.get(), 1);
    }

    #[test]
    fn backend_errors_preserve_category_and_native_code() {
        let error = ContactsError::Backend(
            Error::new(ErrorKind::Platform)
                .with_platform_code(PlatformErrorCode::new(-42).unwrap()),
        );
        assert_eq!(error.kind(), ErrorKind::Platform);
        assert_eq!(error.platform_code().unwrap().get(), -42);
    }

    #[test]
    fn fetch_errors_expose_stable_categories() {
        assert_eq!(
            ContactsError::InvalidIdentifier.kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(ContactsError::NotFound.kind(), ErrorKind::NotFound);
        assert_eq!(
            ContactsError::AuthorizationRequired(ContactsAuthorization::Denied).kind(),
            ErrorKind::PermissionDenied
        );
        assert_eq!(
            ContactsError::AuthorizationRequired(ContactsAuthorization::Unknown).kind(),
            ErrorKind::Unavailable
        );
        assert_eq!(
            ContactsError::AuthorizationRequired(ContactsAuthorization::Limited).platform_code(),
            None
        );
        assert_eq!(
            ContactsError::InvalidContactDraft.kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(ContactsError::InvalidContactDraft.platform_code(), None);
        assert_eq!(
            ContactsError::InvalidContactUpdate.kind(),
            ErrorKind::InvalidInput
        );
        assert_eq!(ContactsError::InvalidContactUpdate.platform_code(), None);
    }

    #[test]
    fn contact_draft_owns_values_and_preserves_optional_labels() {
        let draft = ContactDraft {
            given_name: String::from("Zoë"),
            family_name: String::new(),
            organization_name: String::new(),
            phone_numbers: vec![ContactLabeledValue {
                label: Some(String::from("_$!<Mobile>!$_")),
                value: String::from("+1 555 0100"),
            }],
            email_addresses: vec![ContactLabeledValue {
                label: None,
                value: String::from("zoe@example.test"),
            }],
        };

        assert_eq!(draft.given_name, "Zoë");
        assert_eq!(
            draft.phone_numbers[0].label.as_deref(),
            Some("_$!<Mobile>!$_")
        );
        assert_eq!(draft.phone_numbers[0].value, "+1 555 0100");
        assert_eq!(draft.email_addresses[0].label, None);
        assert_eq!(draft.email_addresses[0].value, "zoe@example.test");
    }

    #[test]
    fn create_extension_is_opt_in_and_facade_forwards_owned_draft() {
        let contacts = Contacts::new(FakeBackend {
            status: ContactsAuthorization::Authorized,
            requests: Rc::new(Cell::new(0)),
        });
        let draft = ContactDraft {
            given_name: String::from("Ada"),
            ..ContactDraft::default()
        };
        assert_eq!(contacts.create_contact(&draft), Ok(()));
        assert_eq!(
            contacts.create_contact(&ContactDraft::default()),
            Err(ContactsError::InvalidContactDraft)
        );
    }

    #[test]
    fn contact_snapshot_owns_common_fields_and_labeled_values() {
        let contact = Contact {
            identifier: String::from("local-id"),
            given_name: String::from("Ada"),
            family_name: String::from("Lovelace"),
            organization_name: String::new(),
            phone_numbers: vec![ContactLabeledValue {
                label: Some(String::from("mobile")),
                value: String::from("+1 555 0100"),
            }],
            email_addresses: vec![ContactLabeledValue {
                label: None,
                value: String::from("ada@example.test"),
            }],
        };

        assert_eq!(contact.identifier, "local-id");
        assert_eq!(contact.given_name, "Ada");
        assert_eq!(contact.family_name, "Lovelace");
        assert!(contact.organization_name.is_empty());
        assert_eq!(contact.phone_numbers[0].label.as_deref(), Some("mobile"));
        assert_eq!(contact.phone_numbers[0].value, "+1 555 0100");
        assert_eq!(contact.email_addresses[0].label, None);
        assert_eq!(contact.email_addresses[0].value, "ada@example.test");
    }
}
