# Async Operation and Ownership Contract

## Runtime-neutral model

`OperationState<T, E>` is a zero-allocation one-shot cell. A backend owns the cell and may start it, publish one success/failure value, or request cancellation. A Rust caller borrows it through one active `OperationFuture`; no executor, task spawn, global scheduler, or process initialization is required.

The observable lifecycle is:

```text
Created -> Running -> Completing -> Completed
    |          |
    +----------+-> CancellationRequested -> Cancelled
```

Completion is accepted only in `Running`. Cancellation may win in `Created` or `Running`. A compare-exchange claims either `Completing` or `CancellationRequested`; all competing completion/cancellation calls then return `false`. The winner publishes the result or cancellation reason and wakes the registered task. Repeated completion cannot replace the first outcome.

`Completion<T, E>` reports `Success(T)`, `Failure(E)`, or `Cancelled(Cancellation)`. Cancellation does not fabricate a backend error value. `CancellationSource` can request a reason; its copyable `CancellationToken` is read-only and reports the reason only after the terminal cancellation state is published.

## Memory ordering and waker slot

The operation phase is an `AtomicU8` with private codes: `Created=0`, `Running=1`, `CancellationRequested=2`, `Completing=3`, `Completed=4`, `Cancelled=5`. The CAS that claims completion/cancellation uses acquire-release ordering. The result is written only by the completion winner, then `Completed` is stored with release ordering; the future uses acquire ordering before it reads the result. The cancellation reason is stored before a release store of `Cancelled`; token readers acquire the phase before reading the reason.

The one-waker slot uses a small `AtomicBool` spin lock around an `UnsafeCell<Option<(u32, Waker)>>`. Lock acquisition is `Acquire`, release is `Release`. Wakers are cloned before lock acquisition; replaced wakers are dropped and ready wakers are invoked after releasing the lock to avoid reentrant code under the lock. The future registers before a second terminal-state check, which closes the completion-versus-registration lost-wake window. Registration IDs wrap while skipping zero; they only identify the current single slot and are not public operation IDs.

Unsafe access is confined to the waker slot and the result cell. The result cell is written by one CAS winner and read by one future consumer after the terminal release/acquire pair. `OperationState<T, E>` is `Sync` only for `T: Send` and `E: Send`.

## Drop and callback lifetime

Dropping `OperationFuture` unregisters its task waker and releases the single active future-consumer claim. It does **not** cancel, destroy, or invalidate the operation. A later future may observe the still-live operation. The backend owner must keep `OperationState` alive until all platform callbacks have stopped; Rust borrowing prevents destruction while a future still borrows the cell, but does not manage an external callback owner's lifetime.

Only one future consumer may be active at a time, and the terminal outcome is delivered at most once. Polling after `Ready` returns `Pending`; callers must follow the standard `Future` rule and not poll again after completion. The completed value is moved out once, and a second future request is rejected after consumption. A dropped pre-completion future leaves the operation and its callback lifetime untouched and permits a later consumer.

No real sleeps are used by tests. A barrier coordinates the completion/cancellation race, and a test waker validates wake behavior and registration teardown.
