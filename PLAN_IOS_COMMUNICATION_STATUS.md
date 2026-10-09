# D44/B49 MessageUI and SharedWithYou status

## D44: capability scope

Rows 081 MessageUI and 082 SharedWithYou remain platform-exclusive status APIs. They have no
portable contract or portable result type. D44 adds no mail, text, account, message delivery,
highlight, or collaboration operation

The two rows expose only Boolean results of Apple's named status APIs. Row 081 maps to
`ios-message-ui-support`; row 082 maps to `ios-shared-with-you-support`. Neither package is re-exported
from a portable module, and no portable contract is part of D44

## B49: iOS API

`ios-message-ui-support` exposes `can_send_mail(MainThreadMarker)` and
`can_send_text(MainThreadMarker)`. These call `MFMailComposeViewController::canSendMail` from iOS
3.0 and `MFMessageComposeViewController::canSendText` from iOS 4.0. Each call returns `false`
below its native API floor and needs a main-thread proof

`ios-shared-with-you-support` exposes
`is_system_collaboration_support_available()`. It calls
`SWHighlightCenter::isSystemCollaborationSupportAvailable` from iOS 16.0 and returns `false`
below that floor

The MessageUI package links `MessageUI.framework`, `UIKit.framework`, Foundation, and Objective-C
runtime. The SharedWithYou package links `SharedWithYou.framework`, Foundation, and Objective-C
runtime. Neither API needs UI, a user prompt, a permission, an entitlement, or a plist key. Neither
status bit proves app-specific access, account state, message delivery, highlight access, or
collaboration success

## Scope limits

No `MFMailComposeViewController` or `MFMessageComposeViewController` instance, presentation,
delegate, content, recipient, account, send, or delivery API is in scope

No `SWHighlightCenter` instance, highlight data, delegate, CloudKit, collaboration metadata,
notice, UI, or account API is in scope

## Validation

`sh platform/ios/ios-message-ui-support/scripts/check.sh` and
`sh platform/ios/ios-shared-with-you-support/scripts/check.sh` are the package gates. They cover
host checks, device and Simulator checks, strict Clippy, rustdoc, source guards, exact direct-link
imports, docs, zero Swift source, and the diff. They do not add or run tests; they build but do not
execute the link probes

Both gates passed in the integrated checkout on Rust 1.94.1 / Xcode 26.6 build 17F113 / iOS SDK
26.5 and are wired in macOS CI. Their arm64 Release probes import only Foundation, the scoped
MessageUI/UIKit or SharedWithYou frameworks, `libSystem.B.dylib`, and `libobjc.A.dylib`; the probes
were built but not executed. No live status query or app/account behavior is claimed

Apple evidence comes from the iOS 26.5 SDK headers for `MFMailComposeViewController`,
`MFMessageComposeViewController`, and `SWHighlightCenter`; generated binding evidence comes from
`objc2-message-ui` 0.3.2 and `objc2-shared-with-you` 0.3.2 with default features off
