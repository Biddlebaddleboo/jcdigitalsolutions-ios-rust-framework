#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    use framework_contacts::Contacts;
    use ios_contacts::IosContactsBackend;

    let contacts = Contacts::new(IosContactsBackend::new());
    let _ = core::hint::black_box(contacts.fetch_contact("selected-local-identifier"));
    let _ = core::hint::black_box(contacts.search_contacts_by_name("selected name", 2));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
