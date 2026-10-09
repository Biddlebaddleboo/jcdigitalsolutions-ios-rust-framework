# `ios-mps-status`

This iOS-only crate maps `MPSGetPreferredDevice(MPSDeviceOptions::Default)` to a `bool`. It returns `true` only when the API returns a non-null MPS-compatible device. The retained native object is dropped inside the call and is not exposed.

The API is available from iOS 12.2. This crate does not create a command queue, submit GPU work, expose device identity or handles, or claim support for any MPS operation, model, or workload. It makes no performance or thread-safety claim. No permission, Info.plist key, or entitlement is required by the queried API.

Run `sh platform/ios/ios-mps-status/check.sh` for package compile, Clippy, rustdoc, feature-isolation, and docs gates. `sh platform/ios/ios-mps-status/check-link-imports.sh` builds the device and simulator link probes and verifies imports. The simulator link probe uses a 14.0 deployment setting; this is a link-check setting, not the API floor.
