# iOS MessageUI status

`ios-message-ui-support` exposes the `can_send_mail` and `can_send_text` status reads through the
public `MFMailComposeViewController.canSendMail()` and `MFMessageComposeViewController.canSendText()`
queries

The first query has an iOS 3.0 API floor; the second has an iOS 4.0 API floor. A call below its
floor returns `false`. Both calls need an `objc2::MainThreadMarker` from the UIKit main thread

These bits report only Apple's point-in-time setup status. They do not prove a later send will
succeed, identify an account, report app-specific access, open compose UI, request permission,
create a message, read recipients or content, or send a message. No callback, notification
observation, account read, plist key, entitlement, or delivery claim is part of this package

The package links `MessageUI.framework`, `UIKit.framework`, and the required Foundation and
Objective-C runtime libraries through `objc2-message-ui` 0.3.2 with default features off and only
the two class bindings plus `objc2-ui-kit` enabled

Apple API references: [canSendMail](https://developer.apple.com/documentation/messageui/mfmailcomposeviewcontroller/cansendmail%28%29),
[canSendText](https://developer.apple.com/documentation/messageui/mfmessagecomposeviewcontroller/cansendtext%28%29)

## Package check gate

Run `sh platform/ios/ios-message-ui-support/scripts/check.sh` from the repository root. The gate
checks the host package, device and Simulator builds, strict Clippy, rustdoc, exact link imports,
source scope, docs, zero Swift source, and the diff. It builds but does not execute the link probes.
The target and link checks need Xcode and the `aarch64-apple-ios` and
`aarch64-apple-ios-sim` Rust targets
