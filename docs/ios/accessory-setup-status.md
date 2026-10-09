# AccessorySetupKit status for iOS

`ios-accessory-setup-status` exposes one operation: an asynchronous count of accessories previously selected for the containing app through AccessorySetupKit

```rust
ios_accessory_setup_status::request_previously_selected_accessory_count(|result| {
    // The callback runs on the main dispatch queue
});
```

This package requires an iOS or iPadOS 18.0+ app deployment target because the typed `objc2-accessory-setup-kit` binding strongly links AccessorySetupKit. The operation creates and activates `ASAccessorySession`, then reads the count of `ASAccessorySession.accessories` only after the `.activated` event. It does not call a picker method, discover nearby accessories, or read names, UUIDs, or accessory objects into Rust. The callback runs once on the main dispatch queue. An invalidated session returns `SessionInvalidated`; non-iOS and Mac Catalyst targets return `NativeApiUnavailable`

The value means only the number of previously selected accessories currently listed for the app. It does not mean any accessory is nearby, connected, available on a transport, or ready for a product operation. It is not a general AccessorySetupKit support or authorization predicate and does not install a persistent event monitor

The host app must configure AccessorySetupKit's information property list keys for its supported accessory types. This crate does not add or validate host plist entries, implement discovery descriptors, display setup UI, establish Bluetooth or Wi-Fi connections, or manage accessory authorization. See [Apple's AccessorySetupKit overview](https://developer.apple.com/documentation/accessorysetupkit), [`ASAccessorySession.accessories`](https://developer.apple.com/documentation/accessorysetupkit/asaccessorysession/accessories), and [Discovering and configuring accessories](https://developer.apple.com/documentation/accessorysetupkit/discovering-and-configuring-accessories)
