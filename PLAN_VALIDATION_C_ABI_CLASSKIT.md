# PLAN_VALIDATION_C_ABI_CLASSKIT.md — G75 ClassKit marker C ABI

## Scope

G75 validates F14's optional C wrapper for D81/B74. The API accepts a caller-owned borrowed opaque
pointer to `NSUserActivity` and writes a Boolean marker result. It does not retain, store, release,
or return the object and does not access assignment or ClassKit store data.

## Gate

`sh bindings/c/check-ios-classkit-deep-link.sh` passed after the root lock refresh. It checks
default/host feature isolation, generated feature closure, locked host/device/Simulator checks,
strict Clippy, Release archives, C11/C++17 compile/link, export/import shape, selector, forbidden
ClassKit-data symbols, and deployment metadata.

Device and Simulator C probes import ClassKit, Foundation, `libSystem.B.dylib`, and
`libobjc.A.dylib`; C++ additionally imports `libc++.1.dylib`. Device minos is iOS 11.3 and Simulator
minos is iOS 14.0. The availability guard runs before pointer dereference. The expected selector is
`isClassKitDeepLink`; no `CLSDataStore`, `CLSContext`, `CLSActivity`, or
`contextIdentifierPath` import evidence was found. No consumer or probe binary was executed.
Local evidence used Rust 1.94.1, Xcode 26.6 build 17F113, and iOS SDK 26.5, below the plan's Xcode
27.x baseline.

## Limits

Compile, link, and symbol checks do not establish a live activity, ClassKit/Schoolwork enrollment,
assignment access, or host threading behavior.
