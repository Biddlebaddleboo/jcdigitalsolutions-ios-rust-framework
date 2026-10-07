# iOS preferences

## Scope and setup

`ios-preferences` implements `framework_preferences::PreferencesBackend` with a caller-owned
`IosPreferences` value around Foundation's `NSUserDefaults.standardUserDefaults`. It adds no
framework-wide registry, executor, callback, or permission prompt. Pass it to
`framework_preferences::Preferences`.

Values are stored under the exact supplied key as Foundation `NSData`, a property-list-compatible
value. Apple documents `NSUserDefaults` as a store for nonsensitive settings and property-list
values ([NSUserDefaults](https://developer.apple.com/documentation/foundation/userdefaults)). Keys
are not prefixed or normalized. A pre-existing value under that key that is not an
`NSData` value returns `InvalidInput` rather than being silently treated as absent.

This is non-secure app preference storage, not Keychain, not cross-device sync, and not an
immediate durable-flush API. `NSUserDefaults` makes a set visible to the process before it stores
the value persistently; persistent storage happens asynchronously. This adapter does not call
`synchronize`, which is not a durability guarantee. No Info.plist key, permission, or entitlement
is required. Apple requires apps and third-party SDKs that use `NSUserDefaults` to declare the
usage reason in `PrivacyInfo.xcprivacy`; this crate does not author the app's privacy manifest.
Use this store for modest settings, not large blobs: the adapter copies bytes into `NSData`, and
Foundation serializes property-list data for its defaults store.

This crate does not declare a deployment target. `NSUserDefaults` is not the source of the iOS 10.0
floor documented for `ios-files`; an app that includes both app-data crates must use iOS 10.0 or
later because the file backend uses public `renameatx_np` and `NSFileManager.temporaryDirectory`
APIs declared available from iOS 10.0 in the Xcode 26.5 SDK headers.

## Semantics and errors

`get_bytes` copies `NSData` into an owned `Vec<u8>`. `set_bytes` copies the borrowed bytes into an
`NSData`; Foundation then stores its property-list value. The adapter reports
`UpdateAtomicity::NotGuaranteed` for every accepted write. If `RequireAtomic` is requested, it
returns `Unsupported` before creating `NSData` or mutating defaults. No multi-key transaction,
crash durability, Keychain secrecy, or sync guarantee is made.

`remove` first asks `objectForKey:` whether the key has a value in `NSUserDefaults`' search list,
then removes the key from the current app's defaults domain. Its boolean is therefore whether a
value was visible before removal, not whether a value existed specifically in the app's persistent
domain. A registered or global-domain fallback can become visible again on a later read after the
app-domain value is removed.

The native `NSUserDefaults` handle is available as a borrow through `IosPreferences::native_defaults`
for APIs outside the portable contract. Native mutations can change values observed through the
portable facade. The object is retained by the backend; the borrowed reference cannot outlive it.

## Thread and linkage behavior

Calls are synchronous and Foundation documents `NSUserDefaults` as thread-safe. The adapter
starts no callbacks or async tasks and has no cancellation operation. Dropping the backend releases
its retained Objective-C reference.

The iOS dependency surface is `objc2` 0.6.5 and `objc2-foundation` 0.3.2 with only the
`NSData`, `NSString`, and `NSUserDefaults` surfaces enabled. A minimal consumer of both app-data
crates was built for device and simulator with
`IPHONEOS_DEPLOYMENT_TARGET=10.0 cargo build --release --target aarch64-apple-ios` and
`IPHONEOS_DEPLOYMENT_TARGET=10.0 cargo build --release --target aarch64-apple-ios-sim`. `otool -L`
showed Foundation, CoreFoundation, `libobjc.A`, `libSystem.B`, and `libiconv.2`; neither binary
imports UIKit, Network, Swift, Python, or another capability framework. This link probe used Xcode
26.6 with the iOS 26.5 SDK, below the planned Xcode 27.x baseline. It proves link imports only; no
simulator launch or live defaults operation was performed.
