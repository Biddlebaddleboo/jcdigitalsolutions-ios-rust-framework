# B83: iOS bookmark resolution

This B83 helper is separate from B82's FileProvider registered-domain count snapshot. It does not
change that API or imply provider readiness. `ios-files::IosResolvedBookmark::resolve_unscoped` maps caller-supplied, non-security-scoped
Foundation bookmark data to one retained file `NSURL` and reports whether Foundation marked the
data stale. It needs iOS 14.2 or later. On older systems it returns `FileError::Backend` with kind
`Unsupported` before the resolver call. The method is `unsafe` because it cannot inspect bookmark
scope and the caller must guarantee that the data is not a security-scoped bookmark.

The call uses `NSURLBookmarkResolutionWithoutUI`, `NSURLBookmarkResolutionWithoutMounting`, and
`NSURLBookmarkResolutionWithoutImplicitStartAccessing`. The last option is available from iOS 14.2
and prevents implicit start for ephemeral security-scoped URLs. Apple's installed iOS header says
this option does not apply to security-scoped bookmark data; the explicit
`NSURLBookmarkResolutionWithSecurityScope` option is unavailable on iOS. The helper cannot detect
scope-bearing bookmark data and does not resolve it safely. The returned URL is not proof of
permission or continued provider availability.

The stale bit is copied to `is_stale()`. The B83 resolver does not make or save a replacement
bookmark. B85 adds `IosPlainBookmarkData::create` for a caller-owned file URL, using
`NSURLBookmarkCreationWithoutImplicitSecurityScope`; its typed value can be resolved safely without
passing arbitrary `NSData` to the unsafe B83 API. Neither helper saves a bookmark or grants access.
A stale bookmark may need replacement through an app-owned flow; this crate does not decide how to
regain user intent or access.

The helper does not show a document picker, check bookmark origin or scope, start or stop a scope,
coordinate I/O, register an `NSFilePresenter`, provide FileProvider lifecycle support, or apply
`IosFiles` sandbox-root containment. The `ios-files` package floor remains iOS 10.0; only this
resolver needs iOS 14.2.

Apple references: [bookmark resolution and stale data](https://developer.apple.com/documentation/foundation/url/init%28resolvingbookmarkdata%3Aoptions%3ArelativeTo%3Abookmarkdataisstale%3A%29-3ic6f),
[security-scoped URLs](https://developer.apple.com/documentation/foundation/nsurl?language=objc), and
[document picker access requirements](https://developer.apple.com/documentation/uikit/uidocumentpickerviewcontroller?changes=_4__7).

See the [B83 bookmark-resolution plan](../../PLAN_IOS_BOOKMARK_RESOLUTION.md) and the separate
[B85 bookmark-creation plan](../../PLAN_IOS_BOOKMARK_CREATION.md), [B82 FileProvider count plan](../../PLAN_IOS_FILEPROVIDER_DOMAIN_COUNT.md), and [D1 app-data plan](../../PLAN_IOS_APP_DATA.md).
