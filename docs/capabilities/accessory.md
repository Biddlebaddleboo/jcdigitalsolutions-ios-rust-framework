# External accessory presence snapshot

`framework-accessory` defines a `no_std` scalar contract for whether a platform's current
connected-and-available accessory list has an entry. The iOS backend is `ios-accessory`.

For iOS, import `ExternalAccessoryBackend` and `IosExternalAccessoryBackend`, then call
`IosExternalAccessoryBackend::connected_accessory_presence()`. The result means only that the
current list returned by the platform API is empty or nonempty at the time of the call. It does
not identify an accessory, establish physical presence for all hardware, establish support for a
specific protocol, or guarantee that communication can begin.

```rust
use framework_accessory::{AccessoryPresenceSnapshot, ExternalAccessoryBackend};
use ios_accessory::IosExternalAccessoryBackend;

let snapshot = IosExternalAccessoryBackend::connected_accessory_presence();
if snapshot == AccessoryPresenceSnapshot::OneOrMoreAvailable {
    // The current platform list contains one or more entries available to this app.
}
```

This is a point-in-time query, not a cached state or notification stream. It does not prompt the
user, open a communication session, access an accessory protocol, or send/receive data. The
contract makes no MFi, entitlement, privacy, or host-metadata claim. See the [iOS adapter
guide](../ios/external-accessory.md) for its precise native scope.
