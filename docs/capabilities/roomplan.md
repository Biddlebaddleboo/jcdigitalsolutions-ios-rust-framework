# RoomPlan device support

`framework-roomplan` defines the portable, allocation-free `RoomPlanDeviceSupport` value. The
value reports a platform support predicate only; it does not represent a scan, room model, sensor
session, camera authorization, or permission state.

On iOS, `ios-roomplan::device_support()` reads Apple's `RoomCaptureSession.isSupported` property
from iOS 16.0. Non-iOS targets return `None`. A supported result indicates framework/device
support, not that a scan has started or will succeed.

See the [iOS backend guide](../ios/roomplan-status.md) for the Swift ABI thunk and validation
boundary.
