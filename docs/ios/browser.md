# iOS external HTTPS URL handler

`ios-browser` is a partial system URL-handler backend, not a Safari UI or general browser API. It
accepts an absolute `framework_format::Uri<'_>`, requires HTTPS and a nonempty Foundation host, and
passes its exact text to `NSURL` before asking UIKit to open it.

```rust
use framework_format::Uri;
use ios_browser::{OpenError, open_external_https_uri};
use ios_runtime::main_thread::MainThread;

fn open_support_page(
    main_thread: MainThread,
    uri: Uri<'_>,
) -> Result<(), OpenError> {
    open_external_https_uri(main_thread, uri, |opened| {
        // `opened` is UIKit's URL-handler result, not a page-load result.
        let _ = opened;
    })
}
```

The caller creates a URI with `Uri::new`, which validates RFC 3986 syntax without normalization,
percent-decoding, DNS lookup, or allocation. This backend rejects non-HTTPS schemes, a missing or
empty authority/host, and URIs that Foundation cannot construct. It does not rewrite the URI. It
does not assert that every syntactically valid HTTPS authority is reachable or meaningful to a web
server.

## Request and callback semantics

The crate calls public UIKit `UIApplication.openURL:options:completionHandler:` with an empty
options dictionary. A synchronous `Ok(())` means only that it issued the system request. The async
completion bool is UIKit's report that the URL was opened by a handler; it does not prove the remote
server responded, that a page loaded, or that a person viewed it. iOS may open the system-selected
browser, another URL handler, or an app associated with a Universal Link. Safari is not guaranteed.
The completion runs asynchronously on the app's main queue. A panic in the Rust callback is caught
and discarded at the Objective-C block boundary.

The function requires `MainThread` and takes no presenter. `UIApplication.sharedApplication` is
unavailable to app extensions. This crate does not use `canOpenURL`, SafariServices,
`SFSafariViewController`, or `WKWebView`; it cannot provide in-app browsing controls, navigation,
page-load status, or browser-dismissal events.

## API floor and configuration

The inspected public `UIApplication.h` in Xcode 26.6 (build 17F113), iOS SDK 26.5, marks
`openURL:options:completionHandler:` available from iOS 10.0. This is the declaration-derived API
floor, not an installed-SDK deployment floor. The SDK `SDKSettings.plist` reports iOS 12.0 as its
minimum deployment target and 26.5 as its default. The repository has no shared deployment target,
and this crate does not set one. The host is below the repository's Xcode 27.x plan baseline.

UIKit and Foundation are the required Apple frameworks; the generated `objc2-ui-kit` and
`objc2-foundation` bindings provide their Rust surface. No Swift source or Swift ABI is used. This
path does not require `LSApplicationQueriesSchemes`: Apple documents that `open` is not subject to
that key's requirement for `canOpenURL`. No permission prompt or entitlement is part of this API
call. No SafariServices binding or system browser UI is included.

Apple references: [`UIApplication.open`](https://developer.apple.com/documentation/uikit/uiapplication/open%28_%3Aoptions%3Acompletionhandler%3A),
[`canOpenURL`](https://developer.apple.com/documentation/uikit/uiapplication/canopenurl%28_%3A%29),
and [`RFC 3986`](https://www.rfc-editor.org/rfc/rfc3986).

Device and simulator checks establish compile/lint evidence only. They do not invoke a live URL
handler, launch a browser, make a network request, establish page-load behavior, or prove which app
handles a URL.
