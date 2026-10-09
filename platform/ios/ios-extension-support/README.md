# ios-extension-support

This iOS-only crate reads `NSExtension.NSExtensionPointIdentifier` from one caller-supplied absolute `.appex` bundle path that the host can access. It uses public `NSBundle`/Foundation metadata APIs, validates the dictionary and string types, and copies the identifier into a Rust-owned `String`.

The query does not call `NSBundle.load`, search for bundles, inspect `.appext` metadata, or establish extension installation, enablement, approval, entitlements, launch, host compatibility, or runtime support. It does not implement App Intents or generate build-host metadata.

`NSBundle.bundleWithURL:` is declared from iOS 4.0. The package uses only the `alloc`, `NSBundle`, `NSDictionary`, `NSString`, and `NSURL` `objc2-foundation` features. The focused check/link scripts build probes but never execute them.
