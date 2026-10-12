use core::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind};

use framework_contacts::{Contact, ContactLabeledValue};
use ios_runtime::main_thread::MainThread;
use objc2::rc::{Allocated, Retained};
use objc2::runtime::{NSObject, ProtocolObject};
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, extern_class,
    extern_methods, extern_protocol, msg_send,
};
use objc2_contacts::{
    CNContact, CNContactEmailAddressesKey, CNContactFamilyNameKey, CNContactGivenNameKey,
    CNContactOrganizationNameKey, CNContactPhoneNumbersKey, CNLabeledValue,
};
use objc2_foundation::{NSArray, NSObjectProtocol, NSString};
use objc2_ui_kit::UIViewController;

use crate::ContactPickerEvent;
use crate::session_policy::PickerCompletion;

/// A main-thread-bound session for selecting one contact with system-owned ContactsUI.
///
/// The caller supplies a visible presenter. This picker presents the system's contacts UI and
/// receives only the user's single selection; it never opens the Contacts database or requests
/// contact authorization. Keep and use the session on the main thread. The callback is one-shot and does
/// not need to be `Send`. Dropping the session detaches the callback and requests dismissal.
pub struct IosContactPickerSession {
    picker: Retained<NativeContactPickerViewController>,
    delegate: Retained<ContactPickerDelegate>,
    _main_thread: MainThreadMarker,
    _not_send: PhantomData<*mut ()>,
}

impl IosContactPickerSession {
    /// Presents a single-contact picker from a caller-owned visible controller.
    ///
    /// The picker displays common name, organization, phone and email properties. A selected
    /// result is copied into an owned [`Contact`] snapshot, including its local Contacts
    /// identifier. The detail card does not display that identifier. UIKit dismisses the picker
    /// after user selection or cancellation; an explicit [`Self::cancel`] requests dismissal.
    /// This method confirms only that UIKit presentation was requested, not that visible UI or
    /// runtime selection occurred. A callback panic is contained at the Objective-C boundary.
    pub fn present<F>(main_thread: MainThread, presenter: &UIViewController, callback: F) -> Self
    where
        F: FnOnce(ContactPickerEvent) + 'static,
    {
        let marker = main_thread.into_objc2();
        let picker = unsafe {
            NativeContactPickerViewController::init(NativeContactPickerViewController::alloc(
                marker,
            ))
        };
        // SAFETY: These public immutable Contacts key constants are valid NSString objects for
        // process lifetime and are the only properties shown in the selected contact's detail card.
        let keys = unsafe {
            NSArray::from_slice(&[
                CNContactGivenNameKey,
                CNContactFamilyNameKey,
                CNContactOrganizationNameKey,
                CNContactPhoneNumbersKey,
                CNContactEmailAddressesKey,
            ])
        };
        picker.setDisplayedPropertyKeys(Some(&keys));
        let delegate = ContactPickerDelegate::new(marker, callback);
        picker.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
        presenter.presentViewController_animated_completion(&picker, true, None);
        Self {
            picker,
            delegate,
            _main_thread: marker,
            _not_send: PhantomData,
        }
    }

    /// Cancels an active session, calls its callback once with [`ContactPickerEvent::Cancelled`],
    /// and requests animated dismissal. Repeated calls after a terminal event do nothing.
    pub fn cancel(&mut self) {
        self.delegate
            .finish(&self.picker, ContactPickerEvent::Cancelled, true);
    }
}

impl Drop for IosContactPickerSession {
    fn drop(&mut self) {
        if self.delegate.ivars().completion.is_pending() {
            self.delegate.ivars().completion.detach();
            self.picker.setDelegate(None);
            self.picker
                .dismissViewControllerAnimated_completion(true, None);
        }
    }
}

struct ContactPickerDelegateIvars {
    completion: PickerCompletion,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - The target stores only main-thread-owned callback state and does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosContactPickerDelegate"]
    #[ivars = ContactPickerDelegateIvars]
    struct ContactPickerDelegate;

    // SAFETY: NSObjectProtocol has no required methods beyond NSObject's implementation.
    unsafe impl NSObjectProtocol for ContactPickerDelegate {}

    // SAFETY: ContactsUI calls these optional delegate methods on the main thread. The session
    // retains the weak delegate target while active; each terminal path detaches before callback,
    // self-retains across app code, and contains Rust callback panics.
    #[allow(non_snake_case)]
    unsafe impl CNContactPickerDelegate for ContactPickerDelegate {
        #[unsafe(method(contactPickerDidCancel:))]
        fn contactPickerDidCancel(&self, picker: &NativeContactPickerViewController) {
            self.finish(picker, ContactPickerEvent::Cancelled, false);
        }

        #[unsafe(method(contactPicker:didSelectContact:))]
        fn contactPicker_didSelectContact(
            &self,
            picker: &NativeContactPickerViewController,
            contact: &CNContact,
        ) {
            let copied = catch_unwind(AssertUnwindSafe(|| contact_snapshot(contact)));
            match copied {
                Ok(contact) => self.finish(picker, ContactPickerEvent::Selected(contact), false),
                Err(_) => self.finish_without_event(picker),
            }
        }
    }
);

impl ContactPickerDelegate {
    fn new(
        marker: MainThreadMarker,
        callback: impl FnOnce(ContactPickerEvent) + 'static,
    ) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(ContactPickerDelegateIvars {
            completion: PickerCompletion::new(callback),
        });
        // SAFETY: NSObject is the direct superclass, and all ivars are initialized before `init`.
        unsafe { msg_send![super(this), init] }
    }

    fn finish(
        &self,
        picker: &NativeContactPickerViewController,
        event: ContactPickerEvent,
        request_dismissal: bool,
    ) {
        let _keep_alive = self.retain();
        if !self.ivars().completion.is_pending() {
            return;
        }
        picker.setDelegate(None);
        if request_dismissal {
            picker.dismissViewControllerAnimated_completion(true, None);
        }
        self.ivars().completion.settle(event);
    }

    fn finish_without_event(&self, picker: &NativeContactPickerViewController) {
        let _keep_alive = self.retain();
        self.ivars().completion.detach();
        picker.setDelegate(None);
    }
}

extern_protocol!(
    /// The public delegate contract for `CNContactPickerViewController`.
    ///
    /// # Safety
    ///
    /// Implementers must preserve the public selector signatures and ContactsUI's main-thread
    /// callback contract.
    #[allow(non_snake_case)]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe trait CNContactPickerDelegate: NSObjectProtocol {
        #[optional]
        #[unsafe(method(contactPickerDidCancel:))]
        fn contactPickerDidCancel(&self, picker: &NativeContactPickerViewController);

        #[optional]
        #[unsafe(method(contactPicker:didSelectContact:))]
        fn contactPicker_didSelectContact(
            &self,
            picker: &NativeContactPickerViewController,
            contact: &CNContact,
        );
    }
);

extern_class!(
    /// The public ContactsUI picker controller; this crate uses only single selection.
    #[unsafe(super(UIViewController))]
    #[thread_kind = MainThreadOnly]
    #[name = "CNContactPickerViewController"]
    struct NativeContactPickerViewController;
);

#[allow(non_snake_case)]
impl NativeContactPickerViewController {
    extern_methods!(
        // SAFETY: Mirrors the public NSObject initializer inherited by this UIKit controller.
        #[unsafe(method(init))]
        #[unsafe(method_family = init)]
        pub unsafe fn init(this: Allocated<Self>) -> Retained<Self>;

        #[unsafe(method(setDisplayedPropertyKeys:))]
        pub fn setDisplayedPropertyKeys(&self, keys: Option<&NSArray<NSString>>);

        #[unsafe(method(setDelegate:))]
        pub fn setDelegate(&self, delegate: Option<&ProtocolObject<dyn CNContactPickerDelegate>>);
    );
}

fn contact_snapshot(contact: &CNContact) -> Contact {
    // SAFETY: The picker is configured to display each property copied below and always returns
    // the selected contact's local identifier.
    let identifier = unsafe { contact.identifier() }.to_string();
    let given_name = unsafe { contact.givenName() }.to_string();
    let family_name = unsafe { contact.familyName() }.to_string();
    let organization_name = unsafe { contact.organizationName() }.to_string();
    let native_phones = unsafe { contact.phoneNumbers() };
    let phone_numbers = (0..native_phones.count())
        .map(|index| {
            let native_value = native_phones.objectAtIndex(index);
            let label = unsafe { native_value.label() }.map(|value| value.to_string());
            let phone = unsafe { native_value.value() };
            let value = unsafe { phone.stringValue() }.to_string();
            ContactLabeledValue { label, value }
        })
        .collect();
    let native_emails = unsafe { contact.emailAddresses() };
    let email_addresses = (0..native_emails.count())
        .map(|index| {
            let native_value: Retained<CNLabeledValue<NSString>> =
                native_emails.objectAtIndex(index);
            let label = unsafe { native_value.label() }.map(|value| value.to_string());
            let value = unsafe { native_value.value() }.to_string();
            ContactLabeledValue { label, value }
        })
        .collect();

    Contact {
        identifier,
        given_name,
        family_name,
        organization_name,
        phone_numbers,
        email_addresses,
    }
}
