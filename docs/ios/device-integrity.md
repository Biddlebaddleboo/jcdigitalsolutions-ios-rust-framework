# iOS DeviceCheck and App Attest support query

`ios-device-integrity::availability_snapshot()` reads only the public DeviceCheck and App Attest `isSupported` values into the portable `framework_device_integrity::AvailabilitySnapshot`.

- DeviceCheck is queried from iOS 11.0; earlier systems report it unsupported
- App Attest is queried from iOS 14.0; iOS 11–13 report App Attest unsupported
- No token/key generation, attestation, assertion, prompt, or network request occurs
- A positive value is not a security result or guarantee of later operation success
- App Attest support may vary by app-extension type
- The binding enables only `DCDevice` and `DCAppAttestService`, not its optional `block2` API

The API floor, feature set, and app-extension caveat are detailed in [`PLAN_IOS_DEVICE_INTEGRITY.md`](../../PLAN_IOS_DEVICE_INTEGRITY.md). The package README contains the exact gate commands.
