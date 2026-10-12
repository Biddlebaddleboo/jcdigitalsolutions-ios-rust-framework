# Custom-render accessibility

`ios_ui::IosCustomRenderAccessibility` attaches an ordered semantic node snapshot to one
`IosCustomRenderHost` `UIImageView`. It does not build widgets or layout. The caller supplies
stable `AccessibilityNodeId` values, text, standard `ios_accessibility::AccessibilityTrait`
values, local frames, and optional activation/custom actions.

Frames use UIKit points relative to the host view's `bounds` origin, not framebuffer pixels or the
parent view's frame. The adapter writes `UIAccessibilityElement.accessibilityFrameInContainerSpace`
and UIKit converts that container-relative rectangle to screen coordinates. UIKit performs that
conversion after view transforms and hierarchy changes; callers should submit frames again after
their own layout changes.

`set_nodes` validates finite coordinates, nonnegative extents, unique node IDs, and unique
node-local action IDs before changing UIKit state. Node order is the supplied slice order. An
unchanged node ID reuses its `UIAccessibilityElement`; updates increment a fixed-width generation,
so actions retained from an older snapshot are rejected. One tree supports at most 4,096 nodes,
64 actions per node, and 256 queued action events. A snapshot update clears queued events from the
prior generation. Queue overflow rejects the newest native action.

UIKit actions do not retain or invoke application callbacks. They append a generation-tagged
`AccessibilityActionEvent` to a bounded Rust queue; the app drains it with `take_action` and runs
its own logic. On iOS 17 and later, default activation uses UIKit's public activation block. On
iOS 12 through 16, the activation label is exposed as a named custom action because the public
activation-block property is only available starting in iOS 17. Custom actions use UIKit's public
target/selector API, available from iOS 8. The app must decide when and how to handle each event.

`request_focus` posts a layout-changed notification for a current node. `notify_screen_changed`
posts a screen-change notification, optionally with a current node. `notify_scrolled` posts the
caller's localized page/status announcement after the app completes its own scrolling; no scroll
behavior is implemented here.

The adapter requires an unclaimed `accessibilityElements` list and reserves it for its lifetime.
Create at most one adapter per host. On drop it closes the action queue, removes the semantic
element list, clears native activation/custom-action references, and restores the host's prior
`isAccessibilityElement` value. The UIKit container is held weakly, action targets hold only weak
Rust queue state, and stale generations cannot enqueue events. The caller remains responsible for
VoiceOver testing and for traits that require behavior; `Adjustable` and `TabBar` traits are
rejected because this adapter does not implement their required native behavior.

The selector/link probe proves public UIKit compilation and linkage only. It does not prove
VoiceOver runtime behavior, focus ordering on-device, or physical-device interoperability.
