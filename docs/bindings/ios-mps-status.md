# iOS MPS preferred-device C API

This opt-in C API asks whether Metal Performance Shaders returns a preferred device with default options. It wraps B68's `preferred_mps_device_available()` query only

`framework_ios_mps_status_preferred_device_available` writes one caller-owned byte. A non-null `out_available` must address valid, properly aligned writable memory for one byte during the synchronous call. The client must prevent unsynchronized concurrent access; the API checks nullness only and does not retain the output address. The wrapper writes zero before platform handling. On iOS, `FRAMEWORK_STATUS_OK` writes exactly zero or one. A true result means only that `MPSGetPreferredDevice(MPSDeviceOptionsDefault)` returned a device at query time. The retained device is dropped before return. A valid non-iOS call returns `FRAMEWORK_STATUS_UNSUPPORTED` with zero output. A null output pointer returns `FRAMEWORK_STATUS_INVALID_ARGUMENT`; a caught Rust panic returns `FRAMEWORK_STATUS_PANIC` with zero output

The API floor is iOS 12.2, and B68 has no runtime availability guard, so call only on iOS 12.2 or later. F24 link probes use minos 12.2 for device and 14.0 for Simulator; the latter is a link setting, not the API floor. The query submits no GPU work, exposes no native device or handle, and does not report operation, model, or workload support. No permission, Info.plist key, or entitlement is required. Runtime cost and thread affinity are not stated; no thread-safety promise is added

F24's optional target-iOS dependency, feature, module/export, ABI entry, and lock edge are wired in the isolated F24 worktree. Root integration must merge those F24-only hunks with concurrent changes, add both F24 gates to macOS CI, update the aggregate binding plan, and link this guide from the binding documentation index. The link gate passed for host, device, and Simulator; linked C/C++ probes were inspected and not executed

See [B68's package plan](../../PLAN_IOS_MPS_STATUS.md), [D62's capability contract](../../PLAN_CAPABILITIES_MPS_STATUS.md), and [G62's validation evidence](../../PLAN_VALIDATION_IOS_MPS_STATUS.md)
