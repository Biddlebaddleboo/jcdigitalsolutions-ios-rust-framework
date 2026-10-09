# G56 — Camera device status validation

The B62 gate validates package compile and import scope only. It does not test camera presence, consent, capture, or runtime parity

## Commands

```sh
sh platform/ios/ios-camera-device-status/check.sh
```

The script runs host, device, and Simulator `cargo check`; strict Clippy; rustdoc; rustfmt; `cargo xtask docs-check`; `cargo xtask zero-swift-source`; and `git diff --check`. It builds device and Simulator link probes but does not run them

## Import criterion

Each linked probe must import only AVFoundation, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`. `nm -u` must show `_objc_getClass`, `_objc_msgSend`, and `AVMediaTypeVideo`; `strings` must show `AVCaptureDevice` and `defaultDeviceWithMediaType:`; no Swift runtime symbol may appear

## Exclusions

- No tests, camera prompt, permission request, session start, or live device check
- No host or simulator execution of the linked probe
- No assertion that a returned device is authorized or ready for capture
- Root integration wires this focused gate into macOS CI and adds the backend/API metadata to row
  046; the 69/113 B/X row-support counts remain unchanged because this extends an existing B row

## Status

Passed in the isolated worktree and integrated checkout on Rust 1.94.1, Xcode 26.6 build 17F113,
and iOS SDK 26.5. Host, device, and Simulator checks, strict Clippy, rustdoc, exact import audits,
docs, zero-Swift, and diff gates passed. Device and Simulator probes were built and inspected, not
executed. No tests or live camera operation were run.
