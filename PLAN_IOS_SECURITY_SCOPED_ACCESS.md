# iOS Security-Scoped URL Access

## Status

`ios-files::IosSecurityScopedAccess` provides a caller-owned RAII guard for the public Foundation
`NSURL.startAccessingSecurityScopedResource` / `stopAccessingSecurityScopedResource` pair. The
installed iOS 26.5 SDK header declares both methods available from iOS 8.0, and the installed
`objc2-foundation` 0.3.2 generated `NSURL` binding exposes typed methods for both. This extends the
crate's existing caller-supplied URL support only by balancing an already-issued scope; it does not
change the crate's iOS 10.0 package floor.

## Contract

- Accept only a file `NSURL`; do not resolve a bookmark, invoke a picker, or manufacture scope.
- Return a guard only when Foundation's start method returns `true`. A `false` result is reported as
  `SecurityScopeStartError::StartRejected`, with no matching stop call.
- Borrow the same `NSURL` for the guard lifetime. The guard is `!Send` and `!Sync`, and its `Drop`
  calls `stopAccessingSecurityScopedResource` exactly once after a successful start.
- Callers keep the guard alive through every operation that needs the scope. Dropping the final
  balanced guard immediately relinquishes this access. Forgetting a guard leaks a scope reference;
  aborting the process cannot run `Drop`.
- The guard performs no file I/O and does not provide URL coordination. External document reads and
  writes still require the relevant file-coordination and FileProvider lifecycle behavior, which
  this crate does not implement.
- A URL string or path does not preserve security scope. The caller must obtain a valid scoped URL,
  such as from a document-picker flow or security-scoped bookmark resolution. This API cannot
  verify that provenance, provider availability, or later read/write success.
- The guard itself adds no prompt, Info.plist usage key, entitlement, global state, runtime, or
  executor. User selection or bookmark resolution remains outside this crate.

## API and availability evidence

- Installed header: `Foundation.framework/Headers/NSURL.h`, `startAccessingSecurityScopedResource`
  and `stopAccessingSecurityScopedResource`, both `API_AVAILABLE(..., ios(8.0), ...)`.
- Installed binding: `objc2-foundation` 0.3.2 generated `NSURL.rs` declares
  `pub unsafe fn startAccessingSecurityScopedResource(&self) -> bool` and
  `pub unsafe fn stopAccessingSecurityScopedResource(&self)` under the `NSURL` feature.
- Apple documents [security-scoped URL access](https://developer.apple.com/documentation/foundation/nsurl?language=objc)
  and the [document picker scope lifecycle](https://developer.apple.com/documentation/uikit/uidocumentpickerviewcontroller?language=objc).

The methods' iOS 8.0 availability is below `ios-files`' existing iOS 10.0 minimum, which is
unchanged because the crate also uses iOS 10.0 APIs. This target check establishes compile-time
binding use only; it does not exercise an app container, a picker, a bookmark, a provider, or a live
security scope.
