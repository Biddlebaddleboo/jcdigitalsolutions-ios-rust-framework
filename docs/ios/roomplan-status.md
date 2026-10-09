# RoomPlan support status

`ios-roomplan::device_support()` returns `Some(RoomPlanDeviceSupport)` on iOS 16.0+ and `None` on
other targets. It calls only `RoomCaptureSession.isSupported`; Apple describes this as device
support, including LiDAR availability. It does not create or run a session, access camera or LiDAR
frames, request permission, present UI, or establish that scanning will succeed.

RoomPlan exposes the property through Swift rather than Objective-C. The backend uses a minimal C
`swiftcall` thunk for the public metadata accessor and getter. A temporary Swift compiler oracle
checks the device and arm64 Simulator lowering, including Clang `swift_context` matching Swift IR's
`swiftself`. No Swift source is checked in or shipped. Release link probes are inspected but never
executed.

Run `sh platform/ios/ios-roomplan/check.sh` on macOS with the iOS device and Simulator Rust targets
installed. See [D56](../../PLAN_CAPABILITIES_ROOMPLAN.md), [B61](../../PLAN_IOS_ROOMPLAN.md), and
[G55](../../PLAN_VALIDATION_IOS_ROOMPLAN.md).
