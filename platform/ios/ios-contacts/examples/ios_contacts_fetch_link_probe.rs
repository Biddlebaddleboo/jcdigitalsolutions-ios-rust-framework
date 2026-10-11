#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    use framework_contacts::{ContactDraft, ContactUpdate, Contacts};
    use ios_contacts::IosContactsBackend;

    let contacts = Contacts::new(IosContactsBackend::new());
    let _ = core::hint::black_box(contacts.fetch_contact("selected-local-identifier"));
    let _ = core::hint::black_box(contacts.fetch_contact_thumbnail("selected-local-identifier"));
    let _ = core::hint::black_box(contacts.export_contact_vcard("selected-local-identifier"));
    let _ = core::hint::black_box(contacts.search_contacts_by_name("selected name", 2));
    let draft = ContactDraft {
        given_name: String::from("Selected"),
        ..ContactDraft::default()
    };
    let _ = core::hint::black_box(contacts.create_contact(&draft));
    let update = ContactUpdate {
        given_name: Some(String::new()),
        ..ContactUpdate::default()
    };
    let _ = core::hint::black_box(contacts.update_contact("selected-local-identifier", &update));
    let _ = core::hint::black_box(contacts.delete_contact("selected-local-identifier"));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
