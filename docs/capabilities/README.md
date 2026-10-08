# Capability support

[`capability-status.json`](capability-status.json) is the canonical machine-readable and
human-readable support manifest. It has 113 capability rows across 15 families. Each row names a
crate/module, portability class, iOS implementation class, verified platform metadata, parity and
performance status, and native-escape status.

| iOS class | Meaning |
| --- | --- |
| `R` | Portable Rust implementation selected after correctness and performance evidence |
| `M` | Rust semantics/state over a minimal public Apple primitive |
| `B` | Apple/system-owned backend reached through Rust |
| `A` | Apple implementation preferred for performance or hardware reasons |
| `C` | Compiler/build/discovery contract |
| `X` | No supported implementation in this workstream; reason is in the row |

Current counts: sixteen rows have `B` support: four partial UIKit example rows, three B1 sandbox-file
and preference rows, one B2 Keychain row, one B3 foreground HTTP row, one B4 local-notification row,
one partial B5 current-location row, one partial B6 plain-text clipboard row, one partial B7
outgoing-share row, one partial B8 accessibility row, one partial B9 acknowledgement-alert row, and
one partial B10 packaged-resource row; 97 rows are `X` for iOS runtime support. The
four D1 portable contracts cover application lifecycle, sandbox files/directories, preferences,
and foreground HTTP values. D7 adds the read-only `framework-resources` contract for exact paths
inside packaged resources. B10 adds an iOS main-bundle backend for exact ordinary files, with an iOS
4.0 API floor; no live read, localization, asset-catalog access, or symlink-containment claim is made.
D5 adds a partial portable plain-text clipboard contract; B6 adds an iOS general-pasteboard backend
with documented iOS privacy behavior and item replacement. No live
privacy prompt or paste behavior is claimed. B1 adds iOS sandbox file and
`NSUserDefaults` backends for three rows.
B2 adds a public Keychain generic-password backend for opaque bytes with explicit protection
requirements; no live Keychain test is claimed. B3 adds the Foundation `URLSession` foreground
HTTP backend; no runtime request or Apple parity test is claimed. D3 adds a partial portable local-
notification contract; B4 adds an iOS local-notification backend but not remote push. D4 adds a
partial portable one-shot location contract; B5 adds an iOS Core Location backend for foreground
current location only, with no continuous updates, geofencing, significant-change monitoring, or
background operation. D6 adds a partial portable outgoing-share contract for UTF-8 text and URL
text; B7 adds UIKit `UIActivityViewController` presentation for those items from an explicit
same-window presenter/source-view context, with an iOS 8.0 floor. No live share UI, recipient
delivery, or unforeseen UIKit presentation recovery is claimed. B8 adds synchronous accessibility
metadata setters for a borrowed `UIView`; its iOS 6.0 API floor is declaration-derived, while this
host's SDK/link deployment minimums are iOS 12.0 for device and iOS 14.0 for simulator. No live
VoiceOver behavior is claimed. B9 adds a synchronous one-action `UIAlertController` acknowledgement
alert with an iOS 9.0 API floor. UIKit's presentation method has no failure callback; no live display
or dismissal is claimed. The UIKit slice remains partial: a Rust-owned app delegate, one window,
basic views, and one target/action callback only.
The manifest counts eight portable contracts as implemented and five as partial.

| Family | Rows | iOS class count |
| --- | ---: | --- |
| Core app/UI | 9 | `B`: 8 (4 partial UIKit example rows; 1 partial clipboard row; 1 partial share row; 1 partial accessibility row; 1 partial acknowledgement-alert row); `X`: 1 |
| Files/data/preferences | 7 | `B`: 4 (1 partial packaged-resource row); `X`: 3 |
| Security/auth | 7 | `B`: 1; `X`: 6 |
| Networking/web | 7 | `B`: 1 partial; `X`: 6 |
| Notifications/background | 6 | `B`: 1 partial; `X`: 5 |
| Sensors/connectivity | 9 | `B`: 1 partial; `X`: 8 |
| Camera/audio/media | 8 | `X`: 8 |
| Graphics/GPU | 9 | `X`: 9 |
| ML/vision/language | 5 | `X`: 5 |
| Personal data/system stores | 8 | `X`: 8 |
| Cloud/accounts/communication | 10 | `X`: 10 |
| Commerce/services | 7 | `X`: 7 |
| Maps/AR/spatial | 5 | `X`: 5 |
| Extension/entitlement capabilities | 12 | `X`: 12 |
| Compiler/build-host capabilities | 4 | `X`: 4 |

For unverified platform metadata the manifest uses `null`, not an inferred empty requirement. A
verified empty list means the row's source documents no required item for that field. The B UI
facts come from [`docs/ios/runtime.md`](../ios/runtime.md) and the `ios-minimal` example; no iOS
minimum version is stated there, so the manifest leaves it unknown. No entitlement, permission, or
framework requirement is inferred for an `X` row. A `true` native-escape value names the handle
documented for that row: UIKit example handles or a capability-specific borrowed native object.
The C7 [App Intents audit](../swift-abi/APP_INTENTS_STAGE0.md) found no stable Rust/C metadata input
or processor API on Xcode 26.6; the Stage 1 runtime path remains unsupported, with no fake capability
API. Re-audit on the Xcode 27.x baseline.

See [the D1 API guide](app-data.md) for the four portable crate contracts and
[the secure-storage guide](secure-storage.md) for D2's opaque-byte contract. These guides detail
ownership, copy, atomicity, async/cancellation, errors, and runtime limits. Later D capability
groups need separate named subplans and executors; D1 and D2 do not claim Workstream D complete.
The D3 [local-notification guide](../notifications.md) describes the portable scheduling contract;
the B4 [iOS guide](../ios/notifications.md) describes the local-only native backend and its runtime
limits. The D4 [location guide](location.md) describes the one-shot portable current-location
contract; the B5 [iOS guide](../ios/location.md) documents its Core Location backend and limits.
The D5 [sharing guide](sharing.md) describes the plain-text clipboard contract; the B6
[iOS guide](../ios/sharing.md) documents the general-pasteboard backend and native privacy limits.
The D6 [share guide](share.md) describes outgoing text and URL-text values; the B7 [iOS
guide](../ios/sharing.md) documents UIKit presentation context, lifecycle, result, and evidence
limits.
The B8 [accessibility guide](../ios/accessibility.md) documents the borrowed-view setters, trait
replacement semantics, API floor, and absence of live VoiceOver evidence.

The optional foreign-language Keychain surface is documented in the
[secure-storage C ABI guide](../bindings/secure-storage.md); it does not change the Rust-native
call path or the iOS capability classification.
