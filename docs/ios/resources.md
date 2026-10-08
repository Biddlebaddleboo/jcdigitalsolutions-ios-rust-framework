# iOS packaged resources

`ios-resources` implements D7's read-only `framework-resources` contract for ordinary files in
`NSBundle.mainBundle()`. Construct `Resources::new(IosResources::new())`, then pass a validated
`ResourcePath` to `Resources::read` or `Resources::read_string`. `IosResources::default()` is an
alias for `IosResources::new()`.

## Lookup and ownership

The backend receives only D7's borrowed `ResourcePath`, splits its exact UTF-8 text at the final
slash, and passes the full final filename (including any dots) with a nil extension and the
preceding relative directory to `NSBundle.URLForResource:withExtension:subdirectory:localization:`.
It passes nil for localization, which Apple's Foundation header documents as lookup of global
resources only. It does not use localized lookup or normalize the path. D7 rejects empty and
absolute paths, empty components, `.` and `..`, backslashes, and NUL bytes before this backend
receives a path.

The resulting `NSURL` stays internal to the backend. Callers cannot supply or receive an arbitrary
native URL or absolute bundle path. A missing bundle URL maps to `ResourceError::Backend` with
`ErrorKind::NotFound`. Otherwise `NSData.dataWithContentsOfURL:options:error:` synchronously loads
the file; this can block. The backend copies `NSData` into a caller-owned `Vec<u8>`. There is no
zero-copy claim. `read_string` uses D7's UTF-8 validation and maps invalid UTF-8 to
`ResourceError::InvalidUtf8` / `ErrorKind::InvalidInput`.

Foundation errors in `NSCocoaErrorDomain` with `NSFileNoSuchFileError` or
`NSFileReadNoSuchFileError` map to `ErrorKind::NotFound`; other native errors map to
`ErrorKind::Platform`. The numeric `NSError.code` is retained when it fits a nonzero `i32`. D7's
`framework_core::Error` carries only `ErrorKind` and an optional `PlatformErrorCode`, so the native
NSError domain is used for classification but is not retained in the portable error.

## Scope and requirements

This backend looks up exact names under the main application's global resource directory only. It
does not enumerate resources, resolve localized resources, access asset catalogs, decode resource
formats, or accept arbitrary URLs. The path validation is lexical; this backend makes no separate
symlink-containment guarantee. Resource lookup is read-only and uses Foundation; it requires no
app permission or entitlement for these bundle-local reads. Compile checks do not exercise a live
application or prove runtime resource packaging behavior.

## Availability evidence

The inspected SDK is Xcode 26.6 (build 17F113) with iOS SDK 26.5. In
`/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/Foundation.framework/Headers/NSBundle.h:72-79`,
Foundation documents that nil localization retrieves global resources, and marks
`URLForResource:withExtension:subdirectory:localization:` as `API_AVAILABLE(..., ios(4.0), ...)`.
This sets the backend API floor at iOS 4.0. The declaration for
`NSData.dataWithContentsOfURL:options:error:` is in the same SDK's `NSData.h:110` and has no higher
availability annotation. The Cocoa missing-file codes used here are declared in
`FoundationErrors.h:11-20`.

The corresponding local `objc2-foundation` 0.3.2 bindings are
`/Users/john/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/objc2-foundation-0.3.2/src/generated/NSBundle.rs:234-243`
and `.../src/generated/NSData.rs:365-371`. The iOS 26.5
`/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/SDKSettings.json`
reports minimum deployment target 12.0, recommended deployment target 15.0, and default deployment
target 26.5. These SDK target values are separate from the Foundation API floor. This crate declares
no deployment target, and no shared repository deployment target is inferred.
