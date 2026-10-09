# framework-vision

`framework-vision` defines the portable `TextRecognitionRevisionSupport` result for one bounded
Vision query. The result records whether the system's `VNRecognizeTextRequest.supportedRevisions`
set contains a caller-supplied revision.

The result does not report text-recognition execution, model readiness, language support, image
content, accuracy, or success on a future image. The iOS query guide is in
[`platform/ios/ios-vision`](../../platform/ios/ios-vision/README.md).
