# ios-map-view

`ios-map-view` owns a main-thread UIKit `MKMapView` and exposes both the prior convenience slice
and low-level native MapKit access. `IosMapView` keeps validated region access, stable marker
snapshots, one bounded route, one bounded polygon, and opt-in foreground user tracking. These
helpers preserve their prior bounds and behavior.

`native_view()` borrows the typed `NativeMapView`; `ui_view()` borrows the same object as `UIView`.
The host owns its parent/child hierarchy, frame, bounds, visibility, constraints, layout, hit testing,
and removal. Native UIKit methods on `UIView` remain available through the borrowed superclass view.

`NativeMapView` adds direct native region, visible map rectangle, camera, overlay, annotation,
selection, renderer lookup, and weak delegate reads. It supports any number of simultaneous native
overlays through repeated `add_overlay` or `add_overlay_at_level` calls; the host owns each retained
overlay handle and removes only overlays it added. The crate re-exports generated `objc2-map-kit`
types for coordinates, geometry, annotations, overlays, cameras, and renderers. Native
`MKOverlayPathRenderer` properties include stroke/fill, line width, and dash pattern setters.

`IosMapViewDelegate` maps the public iOS `MKMapViewDelegate` protocol. It exposes region-change,
annotation-view selection, and nullable overlay-renderer callbacks. `set_delegate` retains the
delegate while it owns MapKit's weak slot; `clear_delegate` and drop clear only that slot if it still
points to this wrapper. Delegate implementations must uphold the Objective-C ABI, avoid panic across
callbacks, and handle synchronous reentrancy. Built-in route/polygon rendering uses its own delegate;
the wrapper rejects a host delegate while either built-in overlay is active and rejects built-in
overlay activation while a host delegate owns the slot.

`MKMapView` is imperative native UIKit, not SwiftUI. MapKit may adjust a requested region or
rectangle to fit the current view size. The native API does not add route calculation, search,
geocoding, directions, offline maps, or a custom renderer. MapKit owns basemap data and any native
network requests. Build/link checks do not prove visible rendering, tile fetch, touch behavior, or
device runtime.

The selected `objc2-map-kit 0.3.2` generated `MKMapView` and `MKAnnotationView` classes are
macOS/AppKit-only, so this crate supplies narrow iOS class declarations for their public UIKit
superclasses. It reuses generated MapKit geometry, camera, annotation, overlay, and renderer types.
The edge-padding methods use UIKit's public `UIEdgeInsets` ABI. No Swift source, private symbols, or
special entitlement is used.

See the [MapKit capability guide](../../../docs/capabilities/mapkit.md), [iOS MapKit guide](../../../docs/ios/mapkit.md), [implementation plan](../../../PLAN_CAPABILITIES_MAPKIT.md), and [registered validation plan](../../../PLAN_VALIDATION_IOS_MAPKIT.md).
