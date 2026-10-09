# Metal device presence

**framework-metal** provides a portable `no_std` contract for a one-time report about whether a backend can obtain a system-default Metal device object. It is not a rendering, compute, or MetalKit API.

```rust
use framework_metal::{MetalDevicePresence, MetalDevicePresenceBackend, MetalDeviceQuery};

fn inspect<B: MetalDevicePresenceBackend>(query: &MetalDeviceQuery<B>) {
    match query.snapshot() {
        MetalDevicePresence::Present => { /* the backend obtained a default device object */ }
        MetalDevicePresence::Absent => { /* the backend reported no default device object */ }
        MetalDevicePresence::Unknown => { /* the backend cannot classify presence */ }
        _ => unreachable!("MetalDevicePresence is non-exhaustive"),
    }
}
```

`MetalDevicePresence` is a copied one-byte semantic value. `MetalDeviceQuery<B>` owns the explicitly supplied static backend; this crate has no platform lookup, global registry, allocation, executor, dynamic dispatch, or hidden initialization. Each synchronous `snapshot` call is a fresh point-in-time backend query.

`Present` means only that the backend's system-default device factory returned a device object. It does not establish that a command queue can be made, a Metal feature is supported, a workload can execute, or any performance target can be met. The portable API exposes no native device handle, command queue, buffer, shader, or view.

This crate does not implement rendering, compute, synchronization, shader compilation, MetalKit, or GPU work submission. The separate [iOS guide](../ios/metal.md) documents its narrow backend.

Portable checks:

```sh
cargo fmt --all -- --check
cargo test -p framework-metal
cargo check -p framework-metal --no-default-features
git diff --check
```
