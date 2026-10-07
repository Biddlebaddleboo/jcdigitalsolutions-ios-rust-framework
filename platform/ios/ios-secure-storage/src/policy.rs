use framework_secure_storage::AccessPolicy;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum KeychainClass {
    AfterFirstUnlock,
    WhenUnlocked,
    AfterFirstUnlockThisDeviceOnly,
    WhenUnlockedThisDeviceOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct MappedPolicy {
    pub(super) class: KeychainClass,
    pub(super) effective: AccessPolicy,
}

pub(super) const fn map_policy(required: AccessPolicy) -> Option<MappedPolicy> {
    let class = match (required.device_unlock_required(), required.device_bound()) {
        (false, false) => KeychainClass::AfterFirstUnlock,
        (true, false) => KeychainClass::WhenUnlocked,
        (false, true) => KeychainClass::AfterFirstUnlockThisDeviceOnly,
        (true, true) => KeychainClass::WhenUnlockedThisDeviceOnly,
    };
    Some(MappedPolicy {
        class,
        effective: required,
    })
}
