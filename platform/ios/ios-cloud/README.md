# `ios-cloud`

This package provides two deliberately separate backends. `IosUbiquityIdentityBackend` collapses
`NSFileManager.defaultManager().ubiquityIdentityToken` immediately to a Boolean presence value; it
never returns, logs, persists, formats, or compares the native token. `IosCloudAccountBackend`
performs a one-shot `CKContainer.accountStatus` query and copies the result/error code to owned
portable values; it does not expose account identity, databases, records, or sync.

The host app must configure the iCloud capability and iCloud Drive Documents service with valid
signing/provisioning, and the device/app iCloud Drive settings must permit access. No
`Info.plist` key is required by the identity backend. The CloudKit account-status query requires
the app's signed CloudKit container and service entitlements but no permission prompt or
CloudKit-specific `Info.plist` usage key. See the [iCloud identity iOS guide](../../../docs/ios/icloud-drive-identity.md),
[the B35 plan](../../../PLAN_IOS_ICLOUD_DRIVE_IDENTITY.md), and [the G29 validation plan](../../../PLAN_VALIDATION_IOS_ICLOUD_DRIVE_IDENTITY.md).

CloudKit behavior, entitlements, callback, and cancellation limits are documented in the
[account-status iOS guide](../../../docs/ios/cloudkit-account-status.md), [B52 plan](../../../PLAN_IOS_CLOUDKIT_ACCOUNT_STATUS.md),
and [G46 validation plan](../../../PLAN_VALIDATION_IOS_CLOUDKIT_ACCOUNT_STATUS.md).

Run `sh platform/ios/ios-cloud/check.sh` after the root workspace lockfile includes these workspace
packages.
