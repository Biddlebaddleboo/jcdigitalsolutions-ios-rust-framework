# iOS RoomPlan support C ABI

The opt-in `ios-roomplan-status` feature exports one synchronous query:

```c
FrameworkStatus framework_ios_roomplan_status_is_supported(uint8_t *out_supported);
```

It calls B61's `RoomCaptureSession.isSupported` path and writes exactly zero or one on
`FRAMEWORK_STATUS_OK`. The required output is caller-owned writable storage for one aligned byte,
initialized to zero before platform handling and not retained. A null output returns
`FRAMEWORK_STATUS_INVALID_ARGUMENT` without a write. A valid non-iOS call returns
`FRAMEWORK_STATUS_UNSUPPORTED` with zero output; a caught Rust panic returns
`FRAMEWORK_STATUS_PANIC` with zero output. The call is synchronous on the caller's thread and
adds no main-thread or broader thread-safety guarantee

The API and required iOS deployment floor are 16.0. B61's strong Swift symbol requires this floor;
there is no weak-symbol fallback. F31's C/C++ link/import gate confirms host imports of only
`libSystem.B.dylib` and device/Simulator imports of `RoomPlan` plus `libSystem.B.dylib`, with minos
16.0 and the required public RoomCaptureSession symbols. The linked consumers are not executed

This query does not create, run, or stop a `RoomCaptureSession`, access camera or LiDAR frames,
request permission, present UI, start a scan, or establish readiness or a successful scan. It adds
no Swift source or new Swift ABI call. See [PLAN_BINDINGS_F31.md](../../PLAN_BINDINGS_F31.md) and
[G115](../../PLAN_VALIDATION_C_ABI_ROOMPLAN.md) for scope and gate limits
