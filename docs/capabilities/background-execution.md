# Background Execution

`framework-background-execution` defines an allocator-free, `no_std` contract for a backend that can begin one finite native background-execution lease. The app receives a lease, polls its backend-specific cooperative expiry signal, and explicitly ends the lease when its bounded work is complete.

```rust,ignore
let execution = BackgroundExecution::new(IosBackgroundExecution::new());
let Some(main_thread) = MainThread::current() else { return; };
let Ok(lease) = execution.begin(main_thread) else { return; };
let expiry = lease.expiry_signal();
while app_has_bounded_work() && !expiry.is_expired() {
    run_one_bounded_unit();
}
lease.end();
```

The example is an app sketch: `app_has_bounded_work` and `run_one_bounded_unit` are app-owned. Platform context requirements differ by backend.

## Contract

- `BackgroundExecution<B>` statically forwards to one `BackgroundExecutionBackend`; it does not select a backend at runtime.
- Each successful `begin` returns one lease. Call `end` promptly when work completes. The native adapter also ends its token when UIKit reports expiry and from `Drop` as a fallback.
- Expiry is a cooperative signal. It does not interrupt Rust work; check between bounded units and stop promptly when it becomes true.
- A successful begin does not provide a time duration, guarantee additional runtime, guarantee continued execution after expiry, schedule a future launch, or guarantee work completion.
- D23 is not BGTaskScheduler app-refresh scheduling, URLSession background transfer, or task history.

The portable crate depends only on `framework-core`. It has no `std`, `alloc`, UIKit, Objective-C, executor, C ABI, global registry, or scheduler dependency.
