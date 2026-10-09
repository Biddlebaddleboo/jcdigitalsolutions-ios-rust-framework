# iOS in-app Safari controller

`ios-safari` implements B64: it creates one public `SFSafariViewController` for a caller-supplied HTTPS
`framework_format::Uri`. It requires `ios_runtime::main_thread::MainThread`. The host owns its
presenting `UIViewController`; `IosSafariViewController::view_controller` exposes a borrowed
`UIViewController` for the host to present. This crate does not decide presentation timing or
dismissal policy.

`Ok(IosSafariViewController)` means only that Foundation and SafariServices constructed the native
controller. It does not establish that UIKit presented the controller, started a request, loaded a
page, or displayed content. The controller is main-thread-only. The wrapper retains it and its
borrowed accessor cannot outlive the wrapper. No Rust delegate or callback is installed; UIKit's
native Safari UI owns navigation and close behavior. The wrapper does not dismiss a presented
controller when dropped.

The URI scheme must be HTTPS and the URI must contain an authority. The backend also requires
`NSURL URLWithString:` to construct a URL with a nonempty host. Foundation may interpret or
canonicalize URI components during conversion; preservation of the original URI's wire form is
not promised. `SFSafariViewController` itself supports both HTTP and HTTPS, but this wrapper
intentionally accepts only HTTPS.

## Browser and privacy boundary

SafariServices owns browser chrome, webpage rendering, transport, cookies, and navigation. The
wrapper does not inspect page contents, history, request or response headers, or cookie state. It
does not use `WKWebView`, add JavaScript, install a delegate, prewarm connections, enable Reader
mode, customize activity controls, download content, or open an external URL handler. It makes no
claim about ATS exceptions, local-network authorization, privacy prompts, extension availability,
Safari app routing, or Safari cookie sharing.

The inspected iOS 26.5 SDK header declares `SFSafariViewController` and `initWithURL:` available
from iOS 9.0. Rust 1.94.1's arm64 iOS device target supports minimum deployment target 10.0; the
crate sets no deployment target. The focused arm64 Simulator link probe uses iOS 14.0. No app,
view, or URL request was run; compile/link checks do not establish Safari runtime behavior or
browser parity.
