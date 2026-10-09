# iOS SharedWithYou system support status

`ios-shared-with-you-support` exposes one status read through
`SWHighlightCenter::isSystemCollaborationSupportAvailable`

Apple defines this bit as full Messages collaboration support for the current software version.
The API has an iOS 16.0 floor; the Rust call returns `false` below that floor

This is a system software-version fact only. It does not report app-specific access, a user's
account or Messages state, permission, highlight visibility, CloudKit access, or collaboration
success. The package does not make a highlight center instance, read highlights, set a delegate,
load collaboration metadata, post a notice, open UI, or prompt the user

The package links `SharedWithYou.framework`, Foundation, and the Objective-C runtime through
`objc2-shared-with-you` 0.3.2 with default features off and only `SWHighlightCenter` enabled. No
`SharedWithYouCore`, CloudKit, UIKit, account, permission, or plist API is in scope

Apple API reference: [SWHighlightCenter](https://developer.apple.com/documentation/sharedwithyou/swhighlightcenter)

## Package check gate

Run `sh platform/ios/ios-shared-with-you-support/scripts/check.sh` from the repository root. The
gate checks the host package, device and Simulator builds, strict Clippy, rustdoc, exact link
imports, source scope, docs, zero Swift source, and the diff. It builds but does not execute the
link probe. The target and link checks need Xcode and the `aarch64-apple-ios` and
`aarch64-apple-ios-sim` Rust targets
