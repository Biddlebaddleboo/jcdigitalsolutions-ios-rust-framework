#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    use framework_contacts::{ContactDraft, Contacts};
    use ios_contacts::IosContactsBackend;

    let contacts = Contacts::new(IosContactsBackend::new());
    let _ = core::hint::black_box(contacts.fetch_contact("selected-local-identifier"));
    let _ = core::hint::black_box(contacts.search_contacts_by_name("selected name", 2));
    let draft = ContactDraft {
        given_name: String::from("Selected"),
        ..ContactDraft::default()
    };
    let _ = core::hint::black_box(contacts.create_contact(&draft));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
