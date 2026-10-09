# G52 — Speech authorization-status gate

## Scope

Gate D53/B58's iOS-only read of `SFSpeechRecognizer::authorizationStatus` and the
`SpeechAuthorizationStatus` value. It does not request permission, create a recognizer, accept
audio, or start recognition.

## Commands

- `sh platform/ios/ios-speech-status/scripts/check.sh`
- `sh platform/ios/ios-speech-status/scripts/check-link-imports.sh`

## Evidence boundary

The package check covers format, host/device/Simulator compilation and strict Clippy, rustdoc,
documentation and zero-Swift gates, API-surface bounds, and device/Simulator Release link-import
inspection. The link probes are built and inspected, never executed. They do not establish live
authorization values, service availability, permission prompt behavior, microphone access, or
recognition output. No tests are added or run by this gate.

## Status

The focused package gate passed in the isolated D53/B58 worktree and again in the integrated root
checkout. Local validation used Rust 1.94.1, Xcode 26.6 build 17F113,
and iPhoneOS/iPhoneSimulator SDK 26.5; the plan's Xcode 27.x baseline is not met.
