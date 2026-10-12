# PLAN_CAPABILITIES_MAPKIT.md — D77/B73 Geometry and B577/B581/B583/B586/B601/B602 MapKit for Row 093

## Objective

Implement portable MapKit geometry and public by-value calls, plus a native iOS map view with validated convenience APIs and a faithful UIKit/MapKit escape surface for row `093-maps-ar-spatial-mapkit`

## Status and recommendation

A partial MapKit slice is implemented in `framework-maps`, `ios-maps`, and `ios-map-view`: caller-supplied coordinate/map-point conversion and distance functions, a UIKit-embeddable `MKMapView`, stable point-marker snapshots, one route and one polygon convenience overlay, opt-in user-location tracking, plus B602 native view composition, viewport/camera, delegate, overlay, renderer, and annotation-selection operations. These slices remain partial and do not promise map-content availability, live callback delivery, or runtime location behavior

The root lock contains `objc2-map-kit` 0.3.2 and the `ios-maps` dependency edges. B577 adds a separate `ios-map-view` edge without widening `ios-maps`' UIKit dependency graph. The native view uses local `objc2` declarations because this selected generated binding exposes `MKMapView` and `MKAnnotationView` only under macOS/AppKit; its MapKit geometry, camera, annotation, overlay, and renderer types use generated public bindings where available. Do not imply this work implements SwiftUI `Map`, an app-specific overlay manager, permission prompting, location-coordinate callbacks, search, geocoding, directions, route calculation, offline map data, or complete MapKit parity

## B577 implementation boundary

- `framework-maps::MapSpan` accepts finite latitude delta in `(0, 180]` and longitude delta in `(0, 360]`; `MapRegion` combines the validated span with `MapCoordinate`
- `ios-map-view::IosMapView` owns one retained, main-thread-bound `MKMapView`; `native_view()` and `ui_view()` return borrowed MapKit/UIKit views for host composition
- `set_region` passes the validated center/span by value to public `setRegion:animated:`; `region` returns `None` if MapKit reports an invalid center or span
- The host remains responsible for size, constraints, and view hierarchy. MapKit may adjust the region to fit the native view's dimensions
- B577 did not enable user location or request authorization. B586 adds only caller-controlled MapKit display/follow after a read-only authorization check; it does not request permission or expose coordinates. B602 adds native overlay/annotation/delegate methods but does not call search, geocoding, directions, or offline-map APIs
- MapKit controls basemap presentation and any content/network requests. Build/link/import validation does not prove tile fetches, visible map rendering, or device behavior
- The inspected Xcode 26.6 / iOS 26.5 public `MKMapView.h` declares `MKMapView : UIView`, `initWithFrame:`, `region`, and `setRegion:animated:`; the class is available from iOS 3.0. The link profile's iOS 12.0 device and iOS 14.0 Simulator floors are validation-probe floors, not API availability claims

## B581 annotation snapshot boundary

- `framework-maps::MapAnnotation<'title>` is a borrowed, caller-owned value with a caller-defined `u64` id, validated `MapCoordinate`, and optional title. The id must be stable across snapshots and unique within each snapshot
- `ios-map-view::IosMapView::replace_annotations` validates duplicate ids before changing MapKit state, preserves the native `MKPointAnnotation` object for ids present in both snapshots, updates its coordinate and copied title, adds new ids, and removes omitted ids
- `MKPointAnnotation` is declared public from iOS 4.0 in `MKPointAnnotation.h`. The Rust crate uses generated `MKPointAnnotation`/`MKAnnotation` types and narrow local add/remove method declarations on the iOS `MKMapView` binding
- The app owns every snapshot and marker meaning. B581 does not define app protocols, custom annotation view construction, subtitle, or icon styling. B602 separately exposes native annotation selection and delegate callbacks; `MKMapView`/MapKit decide visual presentation
- The native view remains main-thread-bound and keeps strong references to active annotation objects. Replacing the snapshot reuses retained objects for stable ids and releases removed objects after calling `removeAnnotation:`
- Host tests prove the portable value and duplicate-id policy; device/Simulator compile, Clippy, Rustdoc, and link/import checks do not prove visible markers, callout interaction, map content, or device runtime behavior

## B583 route overlay boundary

- `IosMapView::replace_route` accepts 2 through 4,096 already validated `MapCoordinate` values, replaces the one route overlay, and rejects invalid point counts before native mutation
- The native path uses public `MKPolyline` and `MKPolylineRenderer` APIs. Its fixed style is `UIColor.blueColor()` at 5 points to preserve the crate's iOS 12 deployment floor; there is no route calculation or caller style surface
- `MKPolyline` is public from iOS 4.0 and `MKPolylineRenderer`/`rendererForOverlay:` from iOS 7.0. The local `objc2` delegate declaration matches the public iOS `MKMapViewDelegate` renderer callback because objc2-map-kit 0.3.2 gates its generated `MKMapView` delegate surface to macOS/AppKit
- `MKMapView.delegate` is a weak singleton slot. Route activation claims it only when empty and returns `MapRouteError::DelegateSlotOccupied` for a host delegate. While a route is active, the wrapper owns that slot and does not forward callbacks; host code must not replace it. `clear_route` and `Drop` remove only this wrapper's route and clear the delegate only if it still points to this wrapper
- Route host tests cover point-count bounds and occupied-slot rejection. The registered iOS profile proves compilation/linkage and symbol selection, not visible route rendering, callback behavior, Simulator execution, or physical-device behavior

## B586 caller-controlled user-location tracking boundary

- `IosMapView::set_user_tracking_mode` accepts `IosUserTrackingMode::{None, Follow, FollowHeading}` and an animation flag. `None` is allowed without authorization and disables both native tracking and `showsUserLocation`
- `Follow` and `FollowHeading` read the current status through the public `CLLocationManager.authorizationStatus` class method. Only existing `AuthorizedWhenInUse` or `AuthorizedAlways` status allows activation. `NotDetermined`, `Restricted`, `Denied`, and unknown values fail closed by disabling tracking/display and return `MapUserTrackingError::LocationAuthorizationRequired`
- The crate never creates a `CLLocationManager`, calls an authorization request API, changes authorization, registers a location delegate, returns location coordinates, or enables background location. The embedding app owns permission UX and must provide `NSLocationWhenInUseUsageDescription` when it opts into this feature. The host handles authorization transitions and reapplies its desired mode
- The public `MKUserTrackingMode`, `userTrackingMode`, and `setUserTrackingMode:animated:` APIs are available from iOS 5.0. The read-only `CLLocationManager.authorizationStatus` class method is available from iOS 4.2. Existing validator deployment floors remain iOS 12.0 device / iOS 14.0 Simulator
- Tracking does not access `MKMapView.delegate`; B583's route behavior remains unchanged, and B601 adds one polygon to the same owned delegate. Host policy tests cover no-permission disable, authorized mode selection, and rejection of all non-authorized/unknown states. The final `ios-map-view` validator profile passed 28/30 gates (0 FAIL, 0 BLOCKED) at snapshot `ff357e70a1fc9a5089098cfdc53bd2c171409ac1189c2f8bbe5ae2babd2d7908`; the two SKIPPED gates are Simulator execution and physical-device proof. Device/Simulator compile, Clippy, Rustdoc, and link/import gates do not prove a permission transition, prompt behavior, location update, visible blue dot, heading rotation, or runtime map behavior

## B601 bounded polygon convenience overlay

- `IosMapView::replace_polygon` accepts 3 through 4,096 validated `MapCoordinate` vertices and `clear_polygon` removes only this wrapper's current polygon. MapKit connects the final vertex to the first; a repeated closing point is not required. The fixed `MKPolygonRenderer` uses a 20%-alpha UIKit blue fill and a 2-point blue stroke. Interior polygons/holes, custom styles, and arbitrary overlays are not part of this convenience API; B602 adds the native MapKit model and renderer surface
- Public `MKPolygon` is available from iOS 4.0 and `MKPolygonRenderer` from iOS 7.0. The package's existing iOS 12.0 device and iOS 14.0 Simulator link floors remain probe floors rather than the API availability claim
- One route and one polygon coexist in a single retained delegate with separate overlay slots. Both activation paths accept only an empty or still wrapper-owned `MKMapView.delegate`; they reject an unrelated host delegate before native mutation. Clearing one shape preserves the delegate while the other remains. Clear/drop remove only the corresponding owned shape and clear the weak slot only after both are absent, and only if the slot still points to this wrapper
- Host regressions cover the three-vertex minimum, 4,096-vertex maximum, over-bound rejection, foreign delegate rejection, and shared delegate lifetime. The registered `ios-map-view` host test gate and device/Simulator link probe include these assertions and the public polygon class, renderer, initializer, fill/stroke selectors. Link evidence does not prove visible map rendering; Simulator execution and physical-device behavior remain unverified

## B602 native UIKit and MapKit host surface

- `IosMapView::ui_view` borrows its native `MKMapView` as `UIView`. The host owns hierarchy, frame/bounds, visibility, layout, hit testing, and removal through UIKit's existing `UIView` methods; no composition runtime or layout cache is added
- `NativeMapView` exposes native region, projected visible-map-rectangle, edge-padding, camera, overlay add/remove/renderer lookup, annotation add/remove/select/deselect, and delegate getter operations. `add_overlay`/`add_overlay_at_level` allow multiple independent native overlays; the application retains their typed native objects and removes only overlays it added
- The crate re-exports generated `objc2-map-kit` geometry, camera, annotation, overlay, polyline, polygon, renderer, and level types. Its generated `MKMapView` class is macOS/AppKit-only, and its generated `MKAnnotationView` class is also macOS-only, so narrow local iOS `UIView` subclass declarations cover only those missing class identities. `UIEdgeInsets` uses UIKit's exact iOS struct encoding for edge-padding selectors; the generated macOS `NSEdgeInsets` signature is not reused
- `IosMapViewDelegate` maps the public iOS `MKMapViewDelegate` protocol subset for region changes, annotation-view selection, and nullable overlay rendering. Safe `set_delegate` retains the Rust delegate while MapKit holds its weak slot; clear/drop nil the slot only if it still points to the owned object. Unsafe protocol implementors must match the ABI, remain main-thread-safe, prevent panic across Objective-C, preserve state lifetime, and account for reentrant callbacks
- Host delegate install rejects an active route/polygon renderer, and route/polygon activation rejects a host-owned delegate. Native overlays remain caller-owned and can use the host's retained delegate. This keeps the existing one-route/one-polygon convenience behavior intact without limiting direct native overlay collections
- Xcode 26.6 / iOS SDK 26.5 public `MKMapView.h` confirms camera/map-rectangle/overlay/selection selectors, `MKMapViewDelegate` callback signatures, `MKAnnotationView : UIView`, and `UIEdgeInsets` for iOS edge padding. Device/Simulator Release link probes prove selector/type availability at their existing iOS 12.0/14.0 floors, not API introduction floors or runtime behavior
- Host policy regressions cover an unowned weak delegate slot, active built-in overlay rejection, and cleanup after delegate replacement. Registered package tests and device/Simulator link checks do not prove callback delivery, annotation touch, viewport animation, visible overlay pixels, rotation, or physical-device behavior

## Implemented contract

- `framework-maps::MapCoordinate::new` accepts only finite latitude in `-90..=90` and longitude in `-180..=180`; the fields are private and getters return the validated degree values
- `framework-maps::MapPoint::new` accepts finite x/y projection values and does not claim they fall inside MapKit's world rectangle
- `framework-maps::MapDistanceMeters::new` accepts finite non-negative meters
- `ios-maps::map_point_for_coordinate`, `coordinate_for_map_point`, and `meters_between_map_points` use the generated MapKit binding functions. They return `None` when iOS 4.0 APIs are unavailable or when native output fails the corresponding finite/range validation
- Non-iOS builds expose these calls as `None`; they do not emulate MapKit geometry
- The unsafe boundary is limited to three generated by-value C calls. The args and results are repr(C) scalar structs; there are no pointers, object handles, callbacks, ownership transfers, or retained state
- No tests or runtime probes were run. Validation is compile/Clippy/docs plus link-only import/symbol inspection

## Implemented API surface

The smallest useful slice is:

- `MKMapPointForCoordinate(CLLocationCoordinate2D)` to project an explicit coordinate
- `MKCoordinateForMapPoint(MKMapPoint)` to recover a coordinate from a projected point
- `MKMetersBetweenMapPoints(MKMapPoint, MKMapPoint)` to ask MapKit for the surface distance between two points

`MKMapPointForCoordinate`, `MKCoordinateForMapPoint`, and `MKMetersBetweenMapPoints` are public declarations in `MapKit.framework/Headers/MKGeometry.h`. The header marks projected geometry as available on iOS 4.0 and marks the conversion/distance functions with `NS_AVAILABLE(..., 4_0)`. The candidate’s minimum iOS API floor is therefore 4.0, not the much newer floor of unrelated MapKit view APIs

`MKMapPoint` is an `x`/`y` value on MapKit’s two-dimensional projection, not a latitude/longitude pair or a meter unit. Apple advises saving coordinate values rather than map points. `MKMetersBetweenMapPoints` reports surface distance in meters and is preferable to a Euclidean distance over projected `x`/`y`

## SDK and generated Rust binding evidence

Inspection used Xcode 26.6 build 17F113, iPhoneOS 26.5 SDK, and Rust/Cargo 1.94.1

- The inspected SDK file is `/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.5.sdk/System/Library/Frameworks/MapKit.framework/Headers/MKGeometry.h`; `MapKit.h` imports it in the public umbrella header
- `MKMapPoint` is a C struct of two `double` fields; the relevant exported functions take and return `MKMapPoint` / `CLLocationCoordinate2D` by value, with no Objective-C object, pointer buffer, callback, or retained handle
- The installed `objc2` 0.6.5 generated-framework catalog at `src/topics/about_generated/list_data.md` lists `objc2-map-kit`
- The generated binding crate is `objc2-map-kit` 0.3.2 and exposes `MKGeometry` types/functions. `MKMapPoint` is `#[repr(C)]` with two `c_double` fields. Its conversion and distance APIs require `MKGeometry` and `objc2-core-location`
- `ios-maps` also has a direct `objc2-core-location` 0.3.2 edge with only `CLLocation` selected to import `CLLocationCoordinate2D`. Root refreshed Cargo.lock with the exact dependency records; no root Cargo.toml or Cargo.lock edit was made by this workstream
- The generated binding marks the coordinate-to-point method and the two C functions unsafe. The package wraps those calls around validated repr(C), by-value scalar structs; it uses no pointer, object, callback, or retention contract

## Configuration, permissions, callbacks, and thread limits

- This geometry-only subset uses `objc2-map-kit` 0.3.2 with default features disabled and only `MKGeometry` plus `objc2-core-location` enabled. `MKGeometry` activates `objc2-foundation/NSValue`; the generated crate's `objc2-core-location` dependency also enables its declared `CLHeading`, `CLLocation`, and `CLPlacemark` binding features. Those are compile-time binding features only and are not called by this adapter
- The generated crate links `MapKit.framework`; its required Core Location dependency contributes `CoreLocation.framework` link metadata. The adapter itself calls no Core Location service API. It does not require a Swift interface, `MKMapView`, or SwiftUI
- The selected functions operate on caller-supplied coordinate values. They do not obtain the user’s location, display a map, access a sensor, or request location authorization; this subset has no location permission prompt or location usage-description requirement
- No MapKit entitlement or Maps capability is documented for these scalar geometry functions in the inspected public header or API docs. This is a statement about the selected value APIs, not every MapKit service
- The selected APIs are synchronous C value calls with no callback, async sequence, view lifecycle, retained object, or delegate. The header assigns no actor or main-thread requirement. Do not extend this to UI-backed APIs or claim a wider undocumented thread-safety guarantee
- `MapCoordinate` bounds are enforced by the portable constructor. The projection may return non-finite map-point values at an edge case; the adapter returns `None` rather than expose those values. Inverse conversion is returned only when MapKit's coordinate passes the same finite latitude/longitude ranges. `MapPoint` inputs are required finite but not constrained to the world rectangle; MapKit determines the projected conversion/distance result
- `MKMapView` is a UIKit view, and Apple specifies that `MKMapViewDelegate` methods run on the app’s main thread. SwiftUI `Map` is a SwiftUI `View` marked `@MainActor`; these are outside the geometry facade
- The original geometry slice does not imply any map-service, network, or location behavior. B577/B586 separately record the host, main-thread, and Core Location authorization contract for the native view

## Geometry acceptance boundary

The geometry-only slice claims only:

- valid caller-supplied latitude/longitude degrees to MapKit projection and back, subject to finite/range validation on returned values
- MapKit’s distance value for two caller-supplied map points
- the iOS 4.0 minimum API floor for these projected-geometry calls
- the exact MapKit framework and generated binding feature requirements above

It must not claim map content availability, data download, user location, permission state, place lookup, geocoding, route computation, custom annotation UI or full MapKit parity. Store geographic coordinates rather than projected points when persistence is required

## Apple and binding sources

- [MapKit framework](https://developer.apple.com/documentation/mapkit)
- [MKMapPoint](https://developer.apple.com/documentation/mapkit/mkmappoint)
- [MKPointAnnotation](https://developer.apple.com/documentation/mapkit/mkpointannotation)
- [MKPolygon](https://developer.apple.com/documentation/mapkit/mkpolygon)
- [MKPolygonRenderer](https://developer.apple.com/documentation/mapkit/mkpolygonrenderer)
- [MKMetersBetweenMapPoints](https://developer.apple.com/documentation/mapkit/mkmappoint/distance%28to%3A%29)
- [MapKit for SwiftUI](https://developer.apple.com/documentation/mapkit/mapkit-for-swiftui)
- [MKMapViewDelegate](https://developer.apple.com/documentation/mapkit/mkmapviewdelegate)
- [objc2-map-kit 0.3.2 crate](https://docs.rs/objc2-map-kit/0.3.2/objc2_map_kit/)
- [objc2-map-kit MKMapPoint binding](https://docs.rs/objc2-map-kit/0.3.2/objc2_map_kit/struct.MKMapPoint.html)
- [objc2-map-kit MKCoordinateForMapPoint binding](https://docs.rs/objc2-map-kit/0.3.2/objc2_map_kit/fn.MKCoordinateForMapPoint.html)
- [objc2-map-kit MKMetersBetweenMapPoints binding](https://docs.rs/objc2-map-kit/0.3.2/objc2_map_kit/fn.MKMetersBetweenMapPoints.html)

## Package-local validation

The package manifest uses these exact iOS dependency edges:

```toml
objc2-core-location = { workspace = true, features = ["CLLocation"] }
objc2-map-kit = { version = "=0.3.2", default-features = false, features = ["MKGeometry", "objc2-core-location"] }
```

`cargo info objc2-map-kit@0.3.2` succeeded and fetched version 0.3.2. The exact binding closure is `MKGeometry` plus `objc2-core-location`; the explicit direct Core Location dependency adds only `CLLocation`. The generated MapKit crate's `objc2-core-location` dependency also enables its declared `CLHeading`, `CLLocation`, and `CLPlacemark` binding features. Those are binding features only and are not invoked here

On Rust/Cargo 1.94.1, `sh platform/ios/ios-maps/check.sh` passed. It checks zero Swift source, formatting, portable no-default-features check/strict Clippy/rustdoc, host iOS-crate check/strict Clippy/rustdoc, iOS device and Simulator check/strict Clippy, the binding feature closure, and the device/Simulator build-only link/import audit. The exact imports were MapKit, CoreLocation, ARKit, Foundation, `libSystem.B.dylib`, and `libobjc.A.dylib`; the link probes contained the three required MapKit C symbols and no Swift, map-view/service, user-location service, camera-session, or UI imports. `minos` was 12.0 for device and 14.0 for Simulator. Those are link-probe targets, not the API floor

`sh -n platform/ios/ios-maps/check.sh platform/ios/ios-maps/check-link-imports.sh` passed. No tests were run and neither link artifact was executed. No location prompt, map rendering, network request, or runtime query was performed
