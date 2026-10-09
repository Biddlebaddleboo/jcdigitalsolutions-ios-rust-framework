# PLAN_CAPABILITIES_ACCESSORY_SETUP.md — B274 AccessorySetupKit count

## Scope

Audit still-X capability row `044-sensors-connectivity-accessorysetupkit` for one useful operation that is distinct from a global support/readiness predicate. Use only public iOS 26.5 SDK declarations and the published typed `objc2-accessory-setup-kit` 0.3.2 binding. Do not present a picker, discover accessories, query live transport state, or expose accessory identities

## Contract

B274 adds `ios-accessory-setup-status::request_previously_selected_accessory_count`, a Rust callback operation for the count of `ASAccessorySession.accessories` after the public `.activated` event. The array contains accessories previously selected for this app. The operation does not claim nearby presence, connection, transport availability, general framework support, or product readiness. Row 044 can be partial (`B`) only for this accessory-list count; root owns aggregate status integration

## API and binding evidence

The installed Xcode 26.6 build `17F113` iPhoneOS SDK 26.5 header `System/Library/Frameworks/AccessorySetupKit.framework/Headers/ASAccessorySession.h` declares:

- `ASAccessorySession.accessories` as a readonly copied `NSArray<ASAccessory *> *`
- `activateWithQueue:eventHandler:` as the operation that activates a session and delivers events on the selected dispatch queue
- separate picker methods `showPickerWithCompletionHandler:` and `showPickerForDisplayItems:completionHandler:`
- `invalidate` to stop operations

Apple's `ASAccessorySession.accessories` documentation defines the property as the app's previously selected accessories. Apple's discovery guide says to read this array after the session activates and describes picker presentation as a separate method. The `objc2-accessory-setup-kit` 0.3.2 binding is present in the local Cargo cache and exposes typed `ASAccessorySession`, `ASAccessoryEventType::Activated`, `accessories`, `activateWithQueue_eventHandler`, and `invalidate` APIs. The implementation uses these Objective-C APIs directly; it does not use Swift ABI, hand-authored selectors, or a Swift wrapper

## Implementation and boundary

The crate creates a session, activates it on the main dispatch queue, reads only `NSArray::len()` inside the activation callback, takes the one-shot completion before invalidating the session, and then invokes that callback. Taking the callback first prevents a reentrant `.invalidated` event from consuming it before the successful result. It never invokes a picker or a discovery operation. The count is read on the callback queue because the generated `objc2` accessor marks `accessories` unsafe with a possible thread-safety requirement. Session invalidation before activation maps to `SessionInvalidated`; non-iOS targets map to `NativeApiUnavailable`

The generated `objc2-accessory-setup-kit` 0.3.2 module uses a strong framework link. The host app therefore must use an iOS or iPadOS 18.0 or later deployment target; Mac Catalyst is unsupported. This package does not claim safe use on an earlier iOS release

The host app remains responsible for AccessorySetupKit `Info.plist` configuration. The callback is asynchronous and runs on the main queue. This API does not deliver future add/remove/change events, retain accessory objects, read names or identifiers, establish a connection, or prove any accessory is physically nearby

## Static validation

`platform/ios/ios-accessory-setup-status/check.sh` runs formatting, host/device/Simulator checks and strict Clippy, rustdoc, documentation checks, shell syntax, and scoped diff checks. No tests, app launch, picker, discovery, runtime manager call, Simulator execution, live device query, or accessory connection is in scope

## Apple primary sources

- [`ASAccessorySession.accessories`](https://developer.apple.com/documentation/accessorysetupkit/asaccessorysession/accessories)
- [`ASAccessorySession`](https://developer.apple.com/documentation/accessorysetupkit/asaccessorysession)
- [Discovering and configuring accessories](https://developer.apple.com/documentation/accessorysetupkit/discovering-and-configuring-accessories)
- [AccessorySetupKit](https://developer.apple.com/documentation/accessorysetupkit)
