# `ios-roomplan`

This crate exposes one non-prompting status query: `device_support()` calls Apple's public
`RoomCaptureSession.isSupported` property and returns its Boolean result as the portable
`RoomPlanDeviceSupport` value. It requires an iOS 16.0-or-newer deployment target because the
RoomPlan framework and property are available from iOS 16.0.

The query does not instantiate or run a `RoomCaptureSession`, access camera or LiDAR frames, show
UI, request permission, or begin a scan. Apple documents `isSupported` as true when the device
contains a LiDAR Scanner. A true result is only framework/device support; it does not guarantee a
successful scan or room model.

RoomPlan exposes this property as a Swift-only static property; the installed SDK has no Objective-C
declaration and this package has no typed `objc2` binding. The backend therefore uses a minimal C
`swiftcall` thunk for the public Swift metadata accessor and getter. The signature and 64-bit
metadata-response layout are checked against a temporary Swift compiler-oracle fixture for device
and Simulator targets. The getter's metadata argument carries Clang's `swift_context` attribute,
which matches Swift IR's `swiftself` parameter. The fixture is created outside the repository by
the gate, never packaged, and neither the gate nor link probe executes an artifact. No Swift source
is shipped.

Run `sh platform/ios/ios-roomplan/check.sh` on macOS with Xcode and the Rust device/Simulator targets
installed. The package gate checks the portable crate, the host fallback, target compilation,
compiler-derived Swift ABI lowering, and Release link imports at the iOS 16.0 API floor.
