# iOS HTTPS web view

`ios-web` creates one native `WKWebView` for a caller-supplied `framework_web::HttpsUrl<'_>`. It
requires a typed `ios_runtime::main_thread::MainThread` proof. `IosWebView::new` creates a default
`WKWebViewConfiguration`, installs its retained navigation delegate, and issues the initial
`NSURLRequest`. It does not wait for a page load or expose a native WebKit handle. Call
`attach_to` with a caller-owned `UIView` and `CGRect` to show the view; dropping `IosWebView` stops
loading, clears its weak delegate, and detaches the child view.

The wrapper exposes only navigation-state snapshots and back, forward, reload, and stop-loading
commands through `framework_web::WebView`. It has no arbitrary URL-load method, file URL or file
read API, HTML/data loading method, `WKUIDelegate`, JavaScript evaluation API, user-content
controller, or script-message handler. WebKit navigation-action and navigation-response policy
callbacks allow HTTPS URLs with a nonempty host and cancel other schemes or nil target frames
(new-window actions). This is a top-level/frame navigation policy, not a network firewall: it does
not inspect every image, script, fetch, or other subresource a page requests, and it does not
restrict navigation to the initial host. A page can initiate later HTTPS navigations. A denied
navigation response may already have caused its server request before WebKit reports the response.

The initial `HttpsUrl` text is passed to Foundation without Rust-side normalization. Foundation
must construct an `NSURL` with HTTPS scheme and a nonempty host. `Ok(IosWebView)` means the native
`loadRequest:` call returned a navigation object; it does not mean the request reached a server,
the page loaded, or content became visible. This slice has no load-completion, failure, progress,
authentication, download, popup, or external-open callback.

## JavaScript and privacy boundary

The backend creates a default `WKWebViewConfiguration`; it does not turn off WebKit's built-in page
behavior or install a Rust/JavaScript bridge. Web content executes according to WebKit's native
defaults, but this crate exposes no JavaScript evaluation or message-channel API. Remote pages can
still send network requests and use WebKit-managed browser features. The default website data store
persists cookies, cache, and other site data to disk; this backend does not provide private-browsing
storage or clear website data. The caller should only load content appropriate for that persistence
and remote-content behavior.

The crate itself does not request a permission or add an entitlement/Info.plist key. It does not
promise that a loaded page cannot invoke operating-system UI or APIs that have their own policy,
privacy prompt, or host-app configuration requirement. It does not claim full Safari/browser parity,
complete subresource transport enforcement, a comprehensive file sandbox, or safe behavior for
untrusted pages.

## API floor and binding note

The inspected public iOS SDK headers in Xcode 26.6 (build 17F113), iOS SDK 26.5, declare
`WKWebView` and the action/response policy enum from iOS 8.0. The current deployment-target
suggestions in that SDK start at iOS 12.0, and its default is iOS 26.5; neither is a crate-selected
minimum. This crate sets no deployment target.

`objc2-web-kit` 0.3.2 is used with default features disabled and only the configuration, navigation,
action, frame, and response features needed here. Its generated `WKWebView` class and
`WKNavigationDelegate` callback methods are gated to macOS, despite the iOS declarations in Apple's
headers. `ios-web` therefore uses objc2's typed `extern_class!`, `extern_methods!`,
`extern_protocol!`, and `define_class!` macros for the iOS class superclass, required selectors,
and the two exact policy callback signatures. The local protocol declaration names Apple's
`WKNavigationDelegate` protocol and uses `NSInteger` for the SDK's `NS_ENUM` block arguments; it
avoids enabling the generated delegate feature and its unrelated authentication/URL-session API
features. The adapter does not call `objc_msgSend` directly or implement WebKit itself. These
declarations are tied to the inspected public header ABI and must be re-audited if the binding or
SDK signature changes.

Apple references: [WKWebView](https://developer.apple.com/documentation/webkit/wkwebview),
[WKNavigationDelegate](https://developer.apple.com/documentation/webkit/wknavigationdelegate), and
[WKWebsiteDataStore](https://developer.apple.com/documentation/webkit/wkwebsitedatastore).

Device and simulator builds establish Rust/API compile and lint evidence only. They do not launch
an app, display web content, send a network request, prove navigation policy at runtime, or verify
site behavior.
