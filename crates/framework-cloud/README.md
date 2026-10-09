# `framework-cloud`

This `no_std`, allocator-free crate contains two platform-independent contracts: the synchronous
`UbiquityIdentitySnapshot`/`UbiquityIdentityBackend` pair for checking whether Foundation returned
a non-`nil` `FileManager.ubiquityIdentityToken`, and `AccountStatusSnapshot`/`CloudAccountBackend`
for a one-shot CloudKit account-status value. Its only workspace dependency is `framework-core`;
it has no Apple-framework or third-party dependency.

The value intentionally excludes the opaque token. `TokenAbsent` does not identify why Foundation
returned `nil`; `TokenPresent` is not proof of ubiquity-container access, sync, or CloudKit account
status. The CloudKit snapshot preserves known semantic states and unknown fixed-width raw values,
with an optional framework-owned error. It does not expose CloudKit data types, account identity,
databases, records, sync, or account-change observation.

See [the portable contract guide](../../docs/capabilities/icloud-drive-identity.md) and
[the named D30 plan](../../PLAN_CAPABILITIES_ICLOUD_DRIVE_IDENTITY.md). See also the
[CloudKit account-status guide](../../docs/capabilities/cloudkit-account-status.md) and
[D47 plan](../../PLAN_CAPABILITIES_CLOUDKIT_ACCOUNT_STATUS.md).
