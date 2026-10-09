# Web view and navigation contract

`framework-web` defines a platform-exclusive, no_std contract for a native web view. It provides a
borrowed `HttpsUrl<'_>`, a back/forward state snapshot, and a small `WebView` control trait. The
contract does not model a portable browser engine: browser rendering, web content, URL loading,
thread requirements, page events, and security enforcement remain native-backend concerns.

`HttpsUrl::new` validates RFC 3986 syntax, requires the `https` scheme and a nonempty authority
host, and preserves the exact caller text. It does not normalize, percent-decode, resolve, rewrite,
or prove that a URI is accepted by a platform URL parser or reachable. `WebViewError` preserves a
portable error category and an optional backend-native code.

`WebView` exposes only current back/forward capability flags and back, forward, reload, and
stop-loading commands. State is a snapshot that can change as native page work proceeds. Commands
do not report whether navigation or page loading succeeds. The contract has no arbitrary-URL load,
file URL, JavaScript, script-message bridge, page-load event, cookie, or browser-UI API.

The iOS backend is documented separately in [`docs/ios/web.md`](../ios/web.md). Other platforms
must add their own bounded native extensions; this contract does not claim that a web view exists
or behaves identically on every platform.
