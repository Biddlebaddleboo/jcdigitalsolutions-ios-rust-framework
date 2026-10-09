# App refresh tasks

`framework-background` gives one portable contract for OS-led app-refresh work. Backend acceptance is not a promise that the OS starts a run

```rust,ignore
use framework_background::{
    AppRefreshBackend, AppRefreshOutcome, AppRefreshRequest, AppRefreshTaskId,
};

fn register_and_submit(backend: &impl AppRefreshBackend) -> framework_core::Result<()> {
    let task_id = AppRefreshTaskId::new("com.example.app.refresh")?;
    let request = AppRefreshRequest::new(task_id.clone());
    backend.register_app_refresh(&task_id, |task| {
        if task.is_expired() {
            return AppRefreshOutcome::Failed;
        }
        AppRefreshOutcome::Succeeded
    })?;
    backend.submit_app_refresh(&request)
}
```

The host app picks the task ID and adds the exact string to native app metadata. Register each ID once per app process before launch ends. The backend holds the launch closure while the ID remains registered; no unregister call exists

The closure is synchronous work, not a `Future`; no async work is awaited. The backend calls the native finish method once after the closure returns. A normal return maps to its `AppRefreshOutcome`, unless expiry wins the atomic race against the finish claim. A Rust panic maps to failure and one finish call

Expiry is a cooperative atomic signal, not forced cancel. Check it during long work, stop promptly, and return `Failed`. The expiry block does not stop app code or finish the native task. If the closure does not return after expiry, the OS may end the app before a finish call

For each accepted request, the OS may launch zero or one run. The OS picks if and when a run can start. A second pending request with the same ID replaces the first; submit a new request to ask for another run. Cancel affects a pending request only

This contract has no executor, global registry, task history, run interval, or durable state. It does not cover URLSession downloads, notification work, UIKit background tasks, or live app behavior
