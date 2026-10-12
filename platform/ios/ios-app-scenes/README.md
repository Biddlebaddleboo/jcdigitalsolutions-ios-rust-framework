# iOS app scene activity and lifecycle

`ios-app-scenes` provides the B81 count-only activity snapshot, the B526 per-window-scene activation snapshot, and the B454 per-scene lifecycle event source. B526 returns exact session identity plus typed activation state and leaves B454 event semantics unchanged. B552 adds an optional main-queue Rust event handler alongside the existing polling queue, with a separate bounded nested-delivery backlog. See [the API guide](../../../docs/ios/app-scenes.md) for examples, identity/overflow behavior, SDK evidence, and limits.

The lifecycle source observes UIKit's six public `UIScene` notifications on the main operation queue, owns/removes its tokens, identifies each event with a Rust-owned copy of the session's UTF-16 persistent identifier, and uses a bounded 32-event drop-oldest queue with a cumulative dropped-event count. It does not replace the host scene delegate or perform UI operations. The package adds no Swift source.

Run `sh platform/ios/ios-app-scenes/check.sh` for iOS device/Simulator compilation, strict Clippy, rustdoc, and actual link/import inspection. Run `cargo +1.94.1 test --locked -p ios-app-scenes` for host mapping, queue, identity, weak-state teardown, and panic-containment regressions. None of these checks delivers a UIKit notification or proves Simulator/device runtime behavior.
