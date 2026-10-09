# iOS Security-Scoped URL Access

`ios-files::IosSecurityScopedAccess` balances one successful Foundation security-scope start with
one stop when the guard drops:

```rust,ignore
let access = IosSecurityScopedAccess::start(&url)?;
coordinator.coordinate_read(&url, options, |coordinated_url| {
    read_external_document(coordinated_url)
})?;
drop(access);
```

Keep the guard alive for the complete period in which any operation needs the scope. Its borrowed
`NSURL` must outlive the guard. The guard is `!Send` and `!Sync`; it starts no worker or callback.
The sample coordinates the operation separately because security scope alone does not coordinate
provider-backed documents.

`start` accepts only a file URL and returns `SecurityScopeStartError::StartRejected` if Foundation
returns `false`. It never calls stop after a rejected start. A successful guard calls stop exactly
once on drop; the last balanced stop immediately revokes access. Do not `mem::forget` the guard.
If a process aborts, Rust destructors do not run.

The input must already be a security-scoped URL, for example one returned from a document picker or
security-scoped bookmark resolution. A path or URL string does not carry that scope. This API cannot
verify the URL's provenance, prompt for access, resolve bookmarks, or guarantee that a later file
operation succeeds. It performs no file I/O, does not add `IosFiles` sandbox-root containment, and
does not provide FileProvider lifecycle support. The existing `ios-files` crate floor remains iOS
10.0; Foundation declares these two methods from iOS 8.0.

See [the focused implementation plan](../../PLAN_IOS_SECURITY_SCOPED_ACCESS.md),
[file coordination](file-coordination.md), and [sandbox files](files.md).

Apple references: [NSURL security-scoped URLs](https://developer.apple.com/documentation/foundation/nsurl?language=objc),
[document picker access](https://developer.apple.com/documentation/uikit/uidocumentpickerviewcontroller?language=objc),
and [the stop-access contract](https://developer.apple.com/documentation/foundation/nsurl/stopaccessingsecurityscopedresource%28%29?language=objc).
