# iOS extension metadata support

`ios-extension-support` reads only `NSExtension.NSExtensionPointIdentifier` from one explicit, caller-supplied absolute `.appex` path. It validates the dictionary and string types and returns a Rust-owned UTF-8 identifier.

This is a runtime metadata read, not extension discovery, registration, installation, approval, entitlement validation, compatibility, launch, or runtime readiness. It does not load extension code, enumerate bundles, read `.appext` metadata, generate a plist, or implement App Intents. Extension-point-specific schema validation remains out of scope.

The backend uses public `NSBundle` and Foundation APIs through `objc2-foundation` 0.3.2. The API floor is iOS 4.0; the focused link probes report minos 12.0 for device and 14.0 for arm64 Simulator. Their host/device/Simulator compile, strict Clippy, rustdoc, feature-closure, and link/import checks passed; probes were inspected but not executed, and no tests were run. See [B77](../../PLAN_IOS_EXTENSION_SUPPORT.md) and [D96](../../PLAN_CAPABILITIES_EXTENSION_BUNDLE_METADATA.md).
