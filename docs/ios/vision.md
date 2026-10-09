# ios-vision

`ios-vision::text_recognition_revision_support(revision)` reports whether the current runtime
lists a caller-supplied revision in `VNRecognizeTextRequest.supportedRevisions`. It returns
`None` below iOS 13.0 and on non-iOS targets.

This status-only slice sends the class property on `VNRecognizeTextRequest` and checks the returned
`NSIndexSet`. It does not create `VNRecognizeTextRequest`, construct a request handler, access an
image, load user data, request permission, show UI, recognize text, or promise model readiness or
recognition success.

The crate uses `objc2-vision` 0.3.2 with defaults disabled and only `VNRequest` plus
`VNRecognizeTextRequest`; `objc2-foundation` enables `NSIndexSet`. See `sh
platform/ios/ios-vision/check.sh` for local non-test checks and the build-only device/Simulator
import gate. That gate inspects linked artifacts but never executes them.
