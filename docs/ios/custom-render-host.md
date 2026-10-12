# Custom render host

`ios-ui::IosCustomRenderHost` embeds one UIKit `UIImageView` in a caller-owned view hierarchy for an application that already owns its rendering model. It is a native presentation primitive, not a widget toolkit or SwiftUI replacement. It does not define layout, state bindings, reconciliation, gesture policy, editor storage, accessibility semantics, or a frame scheduler. Separate opt-in bridges provide [multi-touch](custom-render-touch.md), [text input](custom-render-text-input.md), and [semantic accessibility](custom-render-accessibility.md).

## Frame contract

`Rgba8Frame::new` validates nonzero dimensions, a maximum width and height of 16,384 pixels, a four-byte-aligned row stride of at least `width * 4`, checked byte extents, and a 64 MiB per-frame copy cap. The source format is four bytes per pixel in RGBA order with premultiplied alpha, interpreted in CoreGraphics device RGB. An optional top-left-origin pixel damage rectangle must be nonempty and wholly inside the frame; the current UIKit adapter still replaces and copies the complete image.

`IosCustomRenderHost::present` synchronously copies the validated row extent into `CFData`, then creates a public CoreGraphics image and assigns it to the UIKit image view. It retains no borrowed Rust pointer, submits no asynchronous frame callback, and adds no framework-owned render queue. The image view owns the displayed image until a later frame, `detach`, or host drop. Callers own render scheduling and should submit the newest frame after returning to the foreground. A separate [custom-render text-input bridge](custom-render-text-input.md) attaches native UIKit text input to the host view without changing the framebuffer contract.

The handle is main-thread-affine and `!Send`/`!Sync`. `attach_to`, `set_frame`, `set_hidden`, `geometry`, `present`, and `detach` are UIKit main-thread operations. Drop clears the image and removes the view from its parent. `HostGeometry` reports frame origin, bounds size, content scale, safe-area insets, attached/visible state, and a point-in-time `UIApplicationState`; UIKit's current bounds already reflect orientation and parent layout. For transition events, reuse `ios-app-lifecycle::IosApplicationLifecycle`; defer frame work while inactive/background and submit the newest Rust frame after its foreground event. The host does not install a duplicate lifecycle observer or promise future callbacks itself.

The only renderer dependency is the existing `ios-ui` UIKit package plus its existing workspace CoreGraphics binding. The adapter uses public UIKit, CoreGraphics, and CoreFoundation APIs; it adds no Swift source, C bridge, global runtime, or app-specific rendering semantics.

## Optional Metal presentation

Enable `ios-ui/custom-render-metal` only for an application that wants `IosCustomMetalHost`; the default `ios-ui` dependency graph and CPU host do not enable Metal. The optional host uses public `CAMetalLayer`/Metal APIs and consumes premultiplied BGRA8 frames so framebuffer bytes can be uploaded without a CPU channel-swizzle pass. The source dimensions and optional top-left pixel damage are validated by the same bounded frame policy as the CPU path. Metal updates copy only the damage rectangle to a retained shared texture, then blit it to a matching `BGRA8Unorm` drawable. A new texture, resize, detach, hide/resume, or explicit `invalidate()` requires a full frame before partial updates resume.

`present()` submits the blit but may wait for the prior command before reusing its source texture; `synchronize()` waits for the current command and reports asynchronous errors. These calls are main-thread operations, and a wait may block that thread. This is an explicit, optional path rather than an automatic CPU replacement: use `IosCustomRenderHost` when Metal is unavailable, fails, or does not benefit the workload. Frame scheduling and submit-on-resume behavior remain caller-owned.

The minimal example's `custom-render-metal-demo` feature adds `--custom-metal-render-demo` and a benchmark mode. To build and run the benchmark on an Intel-host Simulator:

```sh
./examples/ios-minimal/build.sh simulator-x86-metal
codesign --force --sign - --timestamp=none target/ios-minimal/simulator-x86-metal/ios-minimal.app
xcrun simctl install booted target/ios-minimal/simulator-x86-metal/ios-minimal.app
xcrun simctl launch --terminate-running-process booted dev.jcdigitalsolutions.ios-minimal --custom-metal-render-benchmark
```

The app writes `tmp/custom-render-metal-benchmark.txt` in its Simulator data container. The harness compares synchronous CPU image presentation with Metal presentation plus completion, using 1179×2556 frames, full and 64×64 damage, two warmups, eight timed submissions, and 30/60/120-Hz requested start pacing. It also records the observed submission cadence; requested pacing is not display refresh or vsync.

One run on the iOS 18.0 iPhone 16 Pro Max x86_64 Simulator produced these mean per-frame wall times:

| Requested Hz | CPU full | Metal full / observed Hz | CPU 64×64 damage | Metal 64×64 damage / observed Hz |
| ---: | ---: | ---: | ---: | ---: |
| 30 | 12.711 ms | 35.470 ms / 28 Hz | 12.647 ms | 130.183 ms / 5 Hz |
| 60 | 9.397 ms | 29.235 ms / 34 Hz | 9.233 ms | 2.348 ms / 56 Hz |
| 120 | 8.576 ms | 28.080 ms / 35 Hz | 8.467 ms | 2.195 ms / 110 Hz |

The 30-Hz Metal sparse case had a 1,014-ms p95 outlier. This single Simulator run shows that full-frame Metal was slower than CPU presentation here, while sparse updates were faster at the two higher requested rates; it does not establish physical-device performance or a general Metal advantage. Metal output was visibly rendered in Simulator. Keep the CPU path available and benchmark on target hardware before selecting a production renderer.

## Runnable example

The `ios-minimal` app paints a 64×32 Rust-generated frame into the host and changes its pixels through a Rust UIKit target/action callback. The custom-render API itself does not depend on the app's label or button. Build and launch on a booted iOS Simulator using the instructions in [the iOS minimal example](../../examples/ios-minimal/README.md). Passing `--exercise-custom-render-host-lifecycle` to the app runs a create → present → update → resize → detach → drop path at launch; the ordinary launch keeps the host visible.

The registered `ios-ui` profile runs host regressions and Release device/Simulator link-import probes at iOS 12.0 and 14.0. The optional Metal feature is included in those link probes only when enabled. Compile and link evidence do not establish visual output, touch delivery, keyboard/IME behavior, or VoiceOver behavior. The CPU and Metal framebuffer output and host lifecycle smoke were observed in Simulator; touch-event injection, live text editing, and VoiceOver interaction remain unverified. No physical-device test is required or claimed.
