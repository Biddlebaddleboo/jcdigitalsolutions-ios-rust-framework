# Portable iCloud Drive identity-presence contract

`framework-cloud` currently exposes a `no_std`, allocator-free value and synchronous static
backend contract for one narrow Foundation observation: whether
`FileManager.ubiquityIdentityToken` was non-`nil` during one call. The result is only
`UbiquityIdentitySnapshot::TokenPresent` or `UbiquityIdentitySnapshot::TokenAbsent`.

The opaque native token never crosses the backend boundary. This API does not return, retain,
compare, serialize, stringify, format, or log the token. It has no account name, account identifier,
persistence, notification observer, or CloudKit operation. Each call is a fresh snapshot and may
become stale immediately after return.

## Meaning and limits

Apple describes the token as an opaque representation of the current user's iCloud Drive Documents
identity. A `nil` value means iCloud is unavailable or no user is logged in; Apple's iCloud Drive
availability guidance also lists an app without the iCloud capability, an app with iCloud Drive
disabled, or a device without a signed-in iCloud account as causes. `TokenAbsent` deliberately
does not distinguish those cases. See [Apple's property documentation](https://developer.apple.com/documentation/foundation/filemanager/ubiquityidentitytoken)
and [Technical Q&A QA1935](https://developer.apple.com/library/archive/qa/qa1935/_index.html).

`TokenPresent` does not establish access to a particular ubiquity container: Apple says reading the
token does not connect the app to its containers; container access requires a separate
`url(forUbiquityContainerIdentifier:)` call. It does not establish file availability, successful
sync, or CloudKit sign-in. Apple explicitly says CloudKit clients must use CloudKit account APIs
such as `accountStatus(completionHandler:)` or `fetchUserRecordID(completionHandler:)`, not this
token. This workstream does not implement those APIs.

No account-change notification is observed, and the snapshot is not a durable account-state cache.
The portable contract does not promise permission prompts, asynchronous completion, or an executor.

## Host configuration

The host app must enable the iCloud capability and select the iCloud Drive Documents service with
valid signing and provisioning. QA1935 also identifies the signed-in device iCloud account and the
per-app iCloud Drive setting as conditions for availability. The adapter itself adds no entitlement,
does not inspect one, and requires no `Info.plist` key. Host entitlement values remain the host
project's signing/configuration responsibility; this contract infers no custom entitlement value
from a token result.

## Scope excluded

CloudKit, container URL lookup or access, file reads or writes, account names or identifiers,
token return/comparison/serialization, sync, notifications, identity-change observation, and
account-state diagnosis remain out of scope.

See [the iOS backend guide](../ios/icloud-drive-identity.md) and the [D30 plan](../../PLAN_CAPABILITIES_ICLOUD_DRIVE_IDENTITY.md).
