# Custom render host

`ios-ui::IosCustomRenderHost` embeds one UIKit `UIImageView` in a caller-owned view hierarchy for an application that already owns its rendering model. It is a native presentation primitive, not a widget toolkit or SwiftUI replacement. It does not define layout, state bindings, reconciliation, gestures, text editing, accessibility semantics, or a frame scheduler.

## Frame contract

`Rgba8Frame::new` validates nonzero dimensions, a maximum width and height of 16,384 pixels, a four-byte-aligned row stride of at least `width * 4`, checked byte extents, and a 64 MiB per-frame copy cap. The source format is four bytes per pixel in RGBA order with premultiplied alpha, interpreted in CoreGraphics device RGB. An optional top-left-origin pixel damage rectangle must be nonempty and wholly inside the frame; the current UIKit adapter still replaces and copies the complete image.

`IosCustomRenderHost::present` synchronously copies the validated row extent into `CFData`, then creates a public CoreGraphics image and assigns it to the UIKit image view. It retains no borrowed Rust pointer, submits no asynchronous frame callback, and adds no framework-owned render queue. The image view owns the displayed image until a later frame, `detach`, or host drop. Callers own render scheduling and should submit the newest frame after returning to the foreground. A separate [custom-render text-input bridge](custom-render-text-input.md) attaches native UIKit text input to the host view without changing the framebuffer contract.

The handle is main-thread-affine and `!Send`/`!Sync`. `attach_to`, `set_frame`, `set_hidden`, `geometry`, `present`, and `detach` are UIKit main-thread operations. Drop clears the image and removes the view from its parent. `HostGeometry` reports frame origin, bounds size, content scale, safe-area insets, attached/visible state, and a point-in-time `UIApplicationState`; UIKit's current bounds already reflect orientation and parent layout. For transition events, reuse `ios-app-lifecycle::IosApplicationLifecycle`; defer frame work while inactive/background and submit the newest Rust frame after its foreground event. The host does not install a duplicate lifecycle observer or promise future callbacks itself.

The only renderer dependency is the existing `ios-ui` UIKit package plus its existing workspace CoreGraphics binding. The adapter uses public UIKit, CoreGraphics, and CoreFoundation APIs; it adds no Swift source, C bridge, global runtime, or app-specific rendering semantics.

## Runnable example

The `ios-minimal` app paints a 64×32 Rust-generated frame into the host and changes its pixels through a Rust UIKit target/action callback. The custom-render API itself does not depend on the app's label or button. Build and launch on a booted iOS Simulator using the instructions in [the iOS minimal example](../../examples/ios-minimal/README.md). Passing `--exercise-custom-render-host-lifecycle` to the app runs a create → present → update → resize → detach → drop path at launch; the ordinary launch keeps the host visible.

The registered `ios-ui` profile runs host regressions and Release device/Simulator link-import probes at iOS 12.0 and 14.0. Compile and link evidence do not establish visual output; only the explicitly reported Simulator run does. No physical-device test is required or claimed.
