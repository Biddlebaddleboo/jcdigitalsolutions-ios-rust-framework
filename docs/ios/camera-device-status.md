# iOS default camera device status

`ios-camera-device-status` exposes `ios_camera_device_status::has_default_video_capture_device() -> bool`

The function asks AVFoundation for the current default `AVMediaTypeVideo` device. It returns `true` if `AVCaptureDevice.defaultDeviceWithMediaType(AVMediaTypeVideo)` returns a device. It returns `false` if no default video device exists or if the crate runs on a non-iOS target

This is a hardware-presence snapshot only. It does not prove consent, capture readiness, session setup, or future access. It does not ask for permission, create `AVCaptureDeviceInput` or `AVCaptureSession`, read samples, or show UI

The query uses the iOS 4.0+ AVFoundation API and links `AVFoundation.framework`. The host app still needs `NSCameraUsageDescription` before any camera access request or input setup

See [D57](../../PLAN_CAPABILITIES_CAMERA_DEVICE_STATUS.md), [B62](../../PLAN_IOS_CAMERA_DEVICE_STATUS.md), and [B62 validation](../../PLAN_VALIDATION_IOS_CAMERA_DEVICE_STATUS.md)
