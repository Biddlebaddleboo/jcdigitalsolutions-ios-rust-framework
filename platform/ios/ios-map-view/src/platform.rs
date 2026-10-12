use core::cell::RefCell;
use core::mem;
use core::ptr::{self, NonNull};
use objc2::define_class;
use objc2::extern_protocol;
use objc2::msg_send;
use objc2::rc::{Allocated, Retained};
use objc2::runtime::{NSObject, NSObjectProtocol, ProtocolObject};
use objc2::{
    AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, Message, extern_class,
    extern_methods,
};
use objc2_core_foundation::CGRect;
use objc2_core_location::CLLocationCoordinate2D;
use objc2_foundation::{NSString, NSUInteger};
use objc2_map_kit::{
    MKAnnotation, MKCoordinateRegion, MKCoordinateSpan, MKMapCamera, MKMapRect, MKOverlay,
    MKOverlayLevel, MKOverlayRenderer, MKPointAnnotation, MKPolygon, MKPolygonRenderer, MKPolyline,
    MKPolylineRenderer, MKUserTrackingMode,
};
use objc2_ui_kit::{UIColor, UIEdgeInsets, UIView};
use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::annotations::sorted_snapshot;
use crate::delegate::{delegate_slot_can_be_claimed, delegate_slot_can_be_cleared};
use crate::polygon::{validate_polygon_delegate_slot, validate_polygon_point_count};
use crate::route::{
    overlay_delegate_is_required, overlay_delegate_slot_is_available, validate_route_point_count,
};
use crate::tracking::{
    IosUserTrackingMode, current_location_authorization, validate_user_tracking_request,
};
use crate::{
    AnnotationSnapshotError, MapAnnotation, MapCoordinate, MapDelegateError, MapPolygonError,
    MapRegion, MapRouteError, MapSpan, MapUserTrackingError,
};

struct AnnotationEntry {
    id: u64,
    native: Retained<MKPointAnnotation>,
}

struct OverlayDelegateState {
    route: RefCell<Option<Retained<MKPolyline>>>,
    polygon: RefCell<Option<Retained<MKPolygon>>>,
}

extern_class!(
    /// Local iOS declaration for the public `MKAnnotationView` UIKit subclass omitted by the selected binding.
    #[unsafe(super(UIView))]
    #[thread_kind = MainThreadOnly]
    #[name = "MKAnnotationView"]
    pub struct NativeMapAnnotationView;
);

extern_protocol!(
    /// The public iOS delegate callbacks needed by native map hosts.
    ///
    /// # Safety
    ///
    /// The protocol name, selectors, argument and return ABI, and main-thread callback behavior
    /// match Apple's public iOS `MKMapView.h` declarations. Implementors must not unwind through
    /// an Objective-C callback, must account for synchronous reentrant map calls, and must keep
    /// callback state valid until the map no longer holds this delegate.
    #[allow(clippy::missing_safety_doc)]
    #[name = "MKMapViewDelegate"]
    pub unsafe trait IosMapViewDelegate: NSObjectProtocol + MainThreadOnly {
        /// Called before MapKit changes the visible region.
        #[optional]
        #[unsafe(method(mapView:regionWillChangeAnimated:))]
        #[unsafe(method_family = none)]
        unsafe fn map_view_region_will_change_animated(
            &self,
            map_view: &NativeMapView,
            animated: bool,
        );

        /// Called after MapKit changes the visible region.
        #[optional]
        #[unsafe(method(mapView:regionDidChangeAnimated:))]
        #[unsafe(method_family = none)]
        unsafe fn map_view_region_did_change_animated(
            &self,
            map_view: &NativeMapView,
            animated: bool,
        );

        /// Called as the visible region changes during a gesture or animation.
        #[optional]
        #[unsafe(method(mapViewDidChangeVisibleRegion:))]
        #[unsafe(method_family = none)]
        unsafe fn map_view_did_change_visible_region(&self, map_view: &NativeMapView);

        /// Called when MapKit selects an annotation view.
        #[optional]
        #[unsafe(method(mapView:didSelectAnnotationView:))]
        #[unsafe(method_family = none)]
        unsafe fn map_view_did_select_annotation_view(
            &self,
            map_view: &NativeMapView,
            view: &NativeMapAnnotationView,
        );

        /// Called when MapKit deselects an annotation view.
        #[optional]
        #[unsafe(method(mapView:didDeselectAnnotationView:))]
        #[unsafe(method_family = none)]
        unsafe fn map_view_did_deselect_annotation_view(
            &self,
            map_view: &NativeMapView,
            view: &NativeMapAnnotationView,
        );

        /// Returns the native renderer for an overlay, or `None` to leave it unrendered.
        #[optional]
        #[unsafe(method(mapView:rendererForOverlay:))]
        #[unsafe(method_family = none)]
        unsafe fn map_view_renderer_for_overlay(
            &self,
            map_view: &NativeMapView,
            overlay: &ProtocolObject<dyn MKOverlay>,
        ) -> Option<Retained<MKOverlayRenderer>>;
    }
);

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - The overlay state is used only by MapKit's main-thread delegate callback.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosMapRouteDelegate"]
    #[ivars = OverlayDelegateState]
    struct OverlayDelegate;

    // SAFETY: NSObject supplies all required NSObjectProtocol methods.
    unsafe impl NSObjectProtocol for OverlayDelegate {}

    // SAFETY: This class conforms to Apple's public MKMapViewDelegate protocol and implements
    // its iOS rendererForOverlay selector with the SDK's exact Objective-C ABI.
    unsafe impl IosMapViewDelegate for OverlayDelegate {
        #[unsafe(method_id(mapView:rendererForOverlay:))]
        unsafe fn map_view_renderer_for_overlay(
            &self,
            _map_view: &NativeMapView,
            overlay: &ProtocolObject<dyn MKOverlay>,
        ) -> Option<Retained<MKOverlayRenderer>> {
            let _keep_alive = self.retain();
            Some(
                catch_unwind(AssertUnwindSafe(|| self.renderer_for_overlay(overlay)))
                    .unwrap_or_else(|_| empty_overlay_renderer(overlay)),
            )
        }
    }
);

impl OverlayDelegate {
    fn new(marker: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(OverlayDelegateState {
            route: RefCell::new(None),
            polygon: RefCell::new(None),
        });
        // SAFETY: This is a fresh NSObject subclass allocation and NSObject's init is valid.
        unsafe { msg_send![super(this), init] }
    }

    fn replace_route(&self, route: Option<Retained<MKPolyline>>) -> Option<Retained<MKPolyline>> {
        core::mem::replace(&mut *self.ivars().route.borrow_mut(), route)
    }

    fn replace_polygon(&self, polygon: Option<Retained<MKPolygon>>) -> Option<Retained<MKPolygon>> {
        core::mem::replace(&mut *self.ivars().polygon.borrow_mut(), polygon)
    }

    fn has_overlays(&self) -> bool {
        let route_active = self.ivars().route.borrow().is_some();
        let polygon_active = self.ivars().polygon.borrow().is_some();
        overlay_delegate_is_required(route_active, polygon_active)
    }

    fn renderer_for_overlay(
        &self,
        overlay: &ProtocolObject<dyn MKOverlay>,
    ) -> Retained<MKOverlayRenderer> {
        let route = self.ivars().route.borrow().clone();
        if let Some(route) = route {
            if overlay_is_route(overlay, &route) {
                // SAFETY: `MKPolylineRenderer` publicly accepts the exact retained `MKPolyline`
                // held by this delegate. MapKit calls this callback on the map's main thread.
                let renderer = unsafe {
                    MKPolylineRenderer::initWithPolyline(MKPolylineRenderer::alloc(), &route)
                };
                let color = UIColor::blueColor();
                // SAFETY: The iOS SDK declares `setStrokeColor:` on MKOverlayPathRenderer with a
                // UIColor object parameter. The local objc2-map-kit binding omits this setter.
                let () = unsafe { msg_send![&*renderer, setStrokeColor: &*color] };
                // SAFETY: The generated public binding matches MKOverlayPathRenderer's CGFloat setter.
                unsafe { renderer.setLineWidth(5.0) };
                // SAFETY: MKPolylineRenderer publicly subclasses MKOverlayRenderer.
                return unsafe { Retained::cast_unchecked(renderer) };
            }
        }

        let polygon = self.ivars().polygon.borrow().clone();
        if let Some(polygon) = polygon {
            if overlay_is_polygon(overlay, &polygon) {
                // SAFETY: `MKPolygonRenderer` publicly accepts the exact retained `MKPolygon`
                // held by this delegate. MapKit calls this callback on the map's main thread.
                let renderer = unsafe {
                    MKPolygonRenderer::initWithPolygon(MKPolygonRenderer::alloc(), &polygon)
                };
                let color = UIColor::blueColor();
                let fill = color.colorWithAlphaComponent(0.2);
                // SAFETY: The iOS SDK declares UIColor object setters on MKOverlayPathRenderer.
                // The local objc2-map-kit binding omits these iOS-only setter declarations.
                let () = unsafe { msg_send![&*renderer, setStrokeColor: &*color] };
                let () = unsafe { msg_send![&*renderer, setFillColor: &*fill] };
                // SAFETY: The generated public binding matches MKOverlayPathRenderer's CGFloat setter.
                unsafe { renderer.setLineWidth(2.0) };
                // SAFETY: MKPolygonRenderer publicly subclasses MKOverlayRenderer.
                return unsafe { Retained::cast_unchecked(renderer) };
            }
        }

        empty_overlay_renderer(overlay)
    }
}

fn overlay_is_route(overlay: &ProtocolObject<dyn MKOverlay>, route: &MKPolyline) -> bool {
    let overlay_pointer = overlay as *const ProtocolObject<dyn MKOverlay> as *const ();
    let route_pointer = route as *const MKPolyline as *const ();
    ptr::eq(overlay_pointer, route_pointer)
}

fn overlay_is_polygon(overlay: &ProtocolObject<dyn MKOverlay>, polygon: &MKPolygon) -> bool {
    let overlay_pointer = overlay as *const ProtocolObject<dyn MKOverlay> as *const ();
    let polygon_pointer = polygon as *const MKPolygon as *const ();
    ptr::eq(overlay_pointer, polygon_pointer)
}

fn empty_overlay_renderer(overlay: &ProtocolObject<dyn MKOverlay>) -> Retained<MKOverlayRenderer> {
    // SAFETY: `overlay` is a live public MKOverlay object supplied by MapKit.
    unsafe { MKOverlayRenderer::initWithOverlay(MKOverlayRenderer::alloc(), overlay) }
}

fn delegate_is_owned(
    current: &ProtocolObject<dyn IosMapViewDelegate>,
    owned: &OverlayDelegate,
) -> bool {
    let current_pointer = current as *const ProtocolObject<dyn IosMapViewDelegate> as *const ();
    let owned_pointer = owned as *const OverlayDelegate as *const ();
    ptr::eq(current_pointer, owned_pointer)
}

fn map_delegate_is_same(
    first: &ProtocolObject<dyn IosMapViewDelegate>,
    second: &ProtocolObject<dyn IosMapViewDelegate>,
) -> bool {
    let first_pointer = first as *const ProtocolObject<dyn IosMapViewDelegate> as *const ();
    let second_pointer = second as *const ProtocolObject<dyn IosMapViewDelegate> as *const ();
    ptr::eq(first_pointer, second_pointer)
}

/// A retained native MapKit view bound to the main thread.
///
/// The type is a UIKit `UIView` subclass and can be embedded in a host controller's view
/// hierarchy. The host remains responsible for sizing and positioning it. MapKit may adjust a
/// requested region to fit the view's current dimensions; `region` reads back the native result.
pub struct IosMapView {
    native: Retained<NativeMapView>,
    annotations: Vec<AnnotationEntry>,
    overlay_delegate: Option<Retained<OverlayDelegate>>,
    host_delegate: Option<Retained<ProtocolObject<dyn IosMapViewDelegate>>>,
    marker: MainThreadMarker,
}

impl IosMapView {
    /// Creates an empty native `MKMapView` on the thread identified by `marker`.
    pub fn new(marker: MainThreadMarker) -> Self {
        // SAFETY: `MKMapView` publicly inherits `UIView` and declares `initWithFrame:`. The
        // zero-sized frame is a valid initial frame; the host assigns layout after embedding.
        let native =
            unsafe { NativeMapView::initWithFrame(NativeMapView::alloc(marker), CGRect::ZERO) };
        Self {
            native,
            annotations: Vec::new(),
            overlay_delegate: None,
            host_delegate: None,
            marker,
        }
    }

    /// Replaces the map's one route path with validated geographic coordinates.
    ///
    /// Paths must contain 2 through [`crate::MAX_ROUTE_COORDINATES`] points. The native path uses
    /// a fixed 5-point UIKit blue stroke. This method claims the map's weak delegate slot only
    /// while it is empty or still owned by this wrapper; an existing host delegate is never
    /// overwritten.
    pub fn replace_route(&mut self, coordinates: &[MapCoordinate]) -> Result<(), MapRouteError> {
        validate_route_point_count(coordinates.len())?;

        let current_delegate = unsafe { self.native.native_delegate() };
        let owns_current = current_delegate.as_ref().is_some_and(|current| {
            self.overlay_delegate
                .as_ref()
                .is_some_and(|owned| delegate_is_owned(current, owned))
        });
        if !overlay_delegate_slot_is_available(current_delegate.is_none(), owns_current) {
            return Err(MapRouteError::DelegateSlotOccupied);
        }

        let mut native_coordinates = native_coordinates(coordinates);
        let point_count = native_coordinates.len() as NSUInteger;
        let points = NonNull::new(native_coordinates.as_mut_ptr())
            .expect("validated route contains at least two coordinates");
        // SAFETY: The bounded, non-empty coordinate buffer matches CLLocationCoordinate2D and
        // MapKit copies the coordinates while creating its owned MKPolyline.
        let route = unsafe { MKPolyline::polylineWithCoordinates_count(points, point_count) };

        let delegate = self
            .overlay_delegate
            .as_ref()
            .cloned()
            .unwrap_or_else(|| OverlayDelegate::new(self.marker));
        if current_delegate.is_none() {
            // SAFETY: The map view is main-thread-bound and this wrapper retains the weak delegate.
            unsafe {
                self.native
                    .set_native_delegate(Some(ProtocolObject::from_ref(&*delegate)))
            };
        }

        if let Some(previous) = delegate.replace_route(Some(route.clone())) {
            let previous: &ProtocolObject<dyn MKOverlay> = ProtocolObject::from_ref(&*previous);
            // SAFETY: Remove only the route overlay owned by this wrapper, on the map's main thread.
            unsafe { self.native.removeOverlay(previous) };
        }
        let route_overlay: &ProtocolObject<dyn MKOverlay> = ProtocolObject::from_ref(&*route);
        // SAFETY: `route` is a retained public MKPolyline and this map is main-thread-bound.
        unsafe { self.native.addOverlay(route_overlay) };
        self.overlay_delegate = Some(delegate);
        Ok(())
    }

    /// Replaces the map's one filled polygon using validated geographic coordinates.
    ///
    /// Polygons must contain 3 through [`crate::MAX_POLYGON_COORDINATES`] vertices. MapKit closes
    /// the final edge; callers do not repeat the first coordinate. The renderer uses a fixed
    /// translucent blue fill and 2-point blue stroke. This method shares the map's owned overlay
    /// delegate with `replace_route`, so one route and one polygon can coexist. A host delegate is
    /// never overwritten.
    pub fn replace_polygon(
        &mut self,
        coordinates: &[MapCoordinate],
    ) -> Result<(), MapPolygonError> {
        validate_polygon_point_count(coordinates.len())?;

        let current_delegate = unsafe { self.native.native_delegate() };
        let owns_current = current_delegate.as_ref().is_some_and(|current| {
            self.overlay_delegate
                .as_ref()
                .is_some_and(|owned| delegate_is_owned(current, owned))
        });
        validate_polygon_delegate_slot(current_delegate.is_none(), owns_current)?;

        let mut native_coordinates = native_coordinates(coordinates);
        let point_count = native_coordinates.len() as NSUInteger;
        let points = NonNull::new(native_coordinates.as_mut_ptr())
            .expect("validated polygon contains at least three coordinates");
        // SAFETY: The bounded, non-empty coordinate buffer matches CLLocationCoordinate2D and
        // MapKit copies the coordinates while creating its owned public MKPolygon.
        let polygon = unsafe { MKPolygon::polygonWithCoordinates_count(points, point_count) };

        let delegate = self
            .overlay_delegate
            .as_ref()
            .cloned()
            .unwrap_or_else(|| OverlayDelegate::new(self.marker));
        if current_delegate.is_none() {
            // SAFETY: The map view is main-thread-bound and this wrapper retains the weak delegate.
            unsafe {
                self.native
                    .set_native_delegate(Some(ProtocolObject::from_ref(&*delegate)))
            };
        }

        let previous = delegate.replace_polygon(Some(polygon.clone()));
        if let Some(previous) = previous {
            // SAFETY: Remove only this wrapper's previous polygon, on the map's main thread.
            unsafe {
                self.native
                    .removeOverlay(ProtocolObject::from_ref(&*previous));
            }
        }
        // SAFETY: `polygon` is a retained public MKPolygon and this map is main-thread-bound.
        unsafe {
            self.native.addOverlay(ProtocolObject::from_ref(&*polygon));
        }
        self.overlay_delegate = Some(delegate);
        Ok(())
    }

    /// Removes the route path without changing marker annotations or an active polygon.
    pub fn clear_route(&mut self) {
        let Some(delegate) = self.overlay_delegate.as_ref().cloned() else {
            return;
        };
        if let Some(route) = delegate.replace_route(None) {
            let route: &ProtocolObject<dyn MKOverlay> = ProtocolObject::from_ref(&*route);
            // SAFETY: Remove only the route overlay owned by this wrapper, on the map's main thread.
            unsafe { self.native.removeOverlay(route) };
        }
        self.release_overlay_delegate_if_unused(&delegate);
    }

    /// Removes the polygon without changing markers or an active route.
    pub fn clear_polygon(&mut self) {
        let Some(delegate) = self.overlay_delegate.as_ref().cloned() else {
            return;
        };
        if let Some(polygon) = delegate.replace_polygon(None) {
            // SAFETY: Remove only this wrapper's polygon, on the map's main thread.
            unsafe {
                self.native
                    .removeOverlay(ProtocolObject::from_ref(&*polygon));
            }
        }
        self.release_overlay_delegate_if_unused(&delegate);
    }

    fn release_overlay_delegate_if_unused(&mut self, delegate: &OverlayDelegate) {
        if delegate.has_overlays() {
            return;
        }
        let current_delegate = unsafe { self.native.native_delegate() };
        if current_delegate
            .as_ref()
            .is_some_and(|current| delegate_is_owned(current, delegate))
        {
            // SAFETY: Clear only this wrapper's weak delegate slot, on the map's main thread.
            unsafe { self.native.set_native_delegate(None) };
        }
        self.overlay_delegate = None;
    }

    /// Sets the map's center and degree span using the public `setRegion:animated:` API.
    ///
    /// The portable region rejects non-finite values and geographic spans outside one world
    /// before this call. `animated` controls only MapKit's native region transition.
    pub fn set_region(&self, region: MapRegion, animated: bool) {
        let native_region = native_region(region);
        // SAFETY: `NativeMapView` is main-thread-only and cannot be moved to another thread.
        // `native_region` contains finite, range-validated coordinates and span deltas matching
        // the public `MKCoordinateRegion` Objective-C struct ABI.
        unsafe { self.native.setRegion_animated(native_region, animated) };
    }

    /// Sets caller-controlled MapKit user-location tracking without requesting authorization.
    ///
    /// `None` is always allowed and disables this map's user-location display. `Follow` and
    /// `FollowHeading` require existing When In Use or Always authorization; on any other status
    /// this method disables tracking and returns an error. The host owns location permission UX
    /// and the `NSLocationWhenInUseUsageDescription` usage string. This method does not modify the
    /// map's overlay delegate.
    pub fn set_user_tracking_mode(
        &self,
        mode: IosUserTrackingMode,
        animated: bool,
    ) -> Result<(), MapUserTrackingError> {
        let native_mode = match mode {
            IosUserTrackingMode::None => {
                self.apply_user_tracking_mode(MKUserTrackingMode::None, animated, false);
                return Ok(());
            }
            IosUserTrackingMode::Follow => MKUserTrackingMode::Follow,
            IosUserTrackingMode::FollowHeading => MKUserTrackingMode::FollowWithHeading,
        };

        let authorization = current_location_authorization();
        if let Err(error) = validate_user_tracking_request(mode, authorization) {
            self.apply_user_tracking_mode(MKUserTrackingMode::None, false, false);
            return Err(error);
        }

        self.apply_user_tracking_mode(native_mode, animated, true);
        Ok(())
    }

    fn apply_user_tracking_mode(
        &self,
        mode: MKUserTrackingMode,
        animated: bool,
        show_user_location: bool,
    ) {
        // SAFETY: This wrapper is main-thread-bound. Both methods are public iOS MKMapView
        // selectors; display is enabled only after checking existing Core Location authorization.
        unsafe {
            self.native.setShowsUserLocation(show_user_location);
            self.native.setUserTrackingMode_animated(mode, animated);
        }
    }

    /// Replaces the caller's complete marker snapshot while preserving native objects for
    /// identifiers that remain present. Duplicate identifiers reject the snapshot before any
    /// native annotation changes occur. MapKit copies each non-null title.
    pub fn replace_annotations(
        &mut self,
        annotations: &[MapAnnotation<'_>],
    ) -> Result<(), AnnotationSnapshotError> {
        let sorted = sorted_snapshot(annotations)?;
        let mut existing = mem::take(&mut self.annotations).into_iter().peekable();
        let mut next = Vec::with_capacity(sorted.len());

        for item in sorted {
            while existing.peek().is_some_and(|entry| entry.id < item.id()) {
                let removed = existing.next().expect("peeked map annotation");
                // SAFETY: The retained object implements the public MKAnnotation protocol, and
                // this map view is main-thread-only.
                unsafe {
                    self.native
                        .removeAnnotation(ProtocolObject::from_ref(&*removed.native))
                };
            }

            let (native, should_add) = if existing.peek().is_some_and(|entry| entry.id == item.id())
            {
                (
                    existing.next().expect("matched map annotation").native,
                    false,
                )
            } else {
                // SAFETY: MKPointAnnotation publicly supports init/new from iOS 4.0 and returns
                // an owned instance implementing MKAnnotation.
                (unsafe { MKPointAnnotation::new() }, true)
            };
            set_annotation_value(&native, *item);

            if should_add {
                // SAFETY: The retained object implements the public MKAnnotation protocol, and
                // this map view is main-thread-only.
                unsafe {
                    self.native
                        .addAnnotation(ProtocolObject::from_ref(&*native))
                };
            }
            next.push(AnnotationEntry {
                id: item.id(),
                native,
            });
        }

        for removed in existing {
            // SAFETY: The retained object implements the public MKAnnotation protocol, and this
            // map view is main-thread-only.
            unsafe {
                self.native
                    .removeAnnotation(ProtocolObject::from_ref(&*removed.native))
            };
        }

        self.annotations = next;
        Ok(())
    }

    /// Reads the current native region, returning `None` if MapKit reports invalid geographic
    /// values.
    pub fn region(&self) -> Option<MapRegion> {
        // SAFETY: `NativeMapView` is main-thread-only and cannot be moved to another thread. The
        // getter returns the public by-value `MKCoordinateRegion` struct.
        let region = unsafe { self.native.native_region() };
        let center = MapCoordinate::new(region.center.latitude, region.center.longitude)?;
        let span = MapSpan::new(region.span.latitudeDelta, region.span.longitudeDelta)?;
        Some(MapRegion::new(center, span))
    }

    /// Borrows the native `MKMapView` for embedding or other UIKit integration on the main
    /// thread. While a route or polygon is active, do not replace its delegate slot through this
    /// escape handle; overlay replacement reports an occupied-slot error if the host changes it.
    pub fn native_view(&self) -> &NativeMapView {
        &self.native
    }

    /// Borrows the same native object as UIKit's `UIView` superclass.
    pub fn ui_view(&self) -> &UIView {
        &self.native
    }

    /// Installs a host delegate and retains it for as long as it owns the native weak slot.
    ///
    /// The wrapper updates the weak slot before releasing the previous delegate. It rejects a delegate
    /// while a built-in route or polygon renderer is active, and never displaces another owner.
    pub fn set_delegate<D>(&mut self, delegate: Retained<D>) -> Result<(), MapDelegateError>
    where
        D: IosMapViewDelegate + Message + 'static,
    {
        let current_delegate = unsafe { self.native.native_delegate() };
        let owns_current = current_delegate.as_ref().is_some_and(|current| {
            self.host_delegate
                .as_ref()
                .is_some_and(|owned| map_delegate_is_same(current, owned))
        });
        let has_builtin_overlays = self
            .overlay_delegate
            .as_ref()
            .is_some_and(|owned| owned.has_overlays());
        if !delegate_slot_can_be_claimed(
            current_delegate.is_none(),
            owns_current,
            has_builtin_overlays,
        ) {
            return Err(MapDelegateError::DelegateSlotOccupied);
        }

        let delegate = ProtocolObject::from_retained(delegate);
        // SAFETY: `delegate` is retained by this wrapper before the map receives its weak slot.
        unsafe {
            self.native
                .set_native_delegate(Some(ProtocolObject::from_ref(&*delegate)))
        };
        self.host_delegate = Some(delegate);
        Ok(())
    }

    /// Clears this wrapper's host delegate without changing a delegate that replaced it.
    pub fn clear_delegate(&mut self) {
        let current_delegate = unsafe { self.native.native_delegate() };
        let owns_current = current_delegate.as_ref().is_some_and(|current| {
            self.host_delegate
                .as_ref()
                .is_some_and(|owned| map_delegate_is_same(current, owned))
        });
        if delegate_slot_can_be_cleared(owns_current) {
            // SAFETY: Clear only the weak slot that still points at this wrapper's retained delegate.
            unsafe { self.native.set_native_delegate(None) };
        }
        self.host_delegate = None;
    }
}

impl Drop for IosMapView {
    fn drop(&mut self) {
        self.clear_route();
        self.clear_polygon();
        self.clear_delegate();
    }
}

fn native_coordinates(coordinates: &[MapCoordinate]) -> Vec<CLLocationCoordinate2D> {
    coordinates
        .iter()
        .map(|coordinate| CLLocationCoordinate2D {
            latitude: coordinate.latitude_degrees(),
            longitude: coordinate.longitude_degrees(),
        })
        .collect()
}

fn set_annotation_value(annotation: &MKPointAnnotation, item: MapAnnotation<'_>) {
    let coordinate = CLLocationCoordinate2D {
        latitude: item.coordinate().latitude_degrees(),
        longitude: item.coordinate().longitude_degrees(),
    };
    let title = item.title().map(NSString::from_str);
    // SAFETY: This call is reached only through a main-thread-bound IosMapView. The coordinate
    // was validated by MapCoordinate, and MKShape copies the optional NSString title.
    unsafe {
        annotation.setCoordinate(coordinate);
        annotation.setTitle(title.as_deref());
    }
}

fn native_region(region: MapRegion) -> MKCoordinateRegion {
    let center = region.center();
    let span = region.span();
    MKCoordinateRegion {
        center: CLLocationCoordinate2D {
            latitude: center.latitude_degrees(),
            longitude: center.longitude_degrees(),
        },
        span: MKCoordinateSpan {
            latitudeDelta: span.latitude_delta_degrees(),
            longitudeDelta: span.longitude_delta_degrees(),
        },
    }
}

extern_class!(
    /// Typed declaration of public iOS `MKMapView`, missing from the selected generated iOS
    /// binding surface in `objc2-map-kit 0.3.2`.
    #[unsafe(super(UIView))]
    #[thread_kind = MainThreadOnly]
    #[name = "MKMapView"]
    pub struct NativeMapView;
);

impl NativeMapView {
    /// Reads MapKit's native coordinate region.
    pub fn region(&self) -> MKCoordinateRegion {
        // SAFETY: The local declaration matches the public by-value region property ABI.
        unsafe { self.native_region() }
    }

    /// Sets a native coordinate region with MapKit's public selector.
    pub fn set_region(&self, region: MKCoordinateRegion, animated: bool) {
        // SAFETY: `MKCoordinateRegion` has the SDK's exact C struct ABI.
        unsafe { self.setRegion_animated(region, animated) };
    }

    /// Reads MapKit's visible rectangle in projected map coordinates.
    pub fn visible_map_rect(&self) -> MKMapRect {
        // SAFETY: Matches the public MKMapRect property ABI.
        unsafe { self.visibleMapRect() }
    }

    /// Sets the visible projected map rectangle.
    pub fn set_visible_map_rect(&self, rect: MKMapRect, animated: bool) {
        // SAFETY: `MKMapRect` matches the public MapKit by-value struct ABI.
        unsafe { self.setVisibleMapRect_animated(rect, animated) };
    }

    /// Returns the projected rectangle adjusted to this map view's aspect ratio.
    pub fn map_rect_that_fits(&self, rect: MKMapRect) -> MKMapRect {
        // SAFETY: `MKMapRect` matches the public MapKit by-value struct ABI.
        unsafe { self.mapRectThatFits(rect) }
    }

    /// Sets the visible projected rectangle with UIKit edge padding.
    pub fn set_visible_map_rect_with_edge_padding(
        &self,
        rect: MKMapRect,
        padding: UIEdgeInsets,
        animated: bool,
    ) {
        // SAFETY: The iOS SDK declares `UIEdgeInsets` for this selector; the local type preserves
        // its `{UIEdgeInsets=CGFloat,CGFloat,CGFloat,CGFloat}` encoding.
        unsafe { self.setVisibleMapRect_edgePadding_animated(rect, padding, animated) };
    }

    /// Returns the projected rectangle adjusted for aspect ratio and UIKit edge padding.
    pub fn map_rect_that_fits_with_edge_padding(
        &self,
        rect: MKMapRect,
        padding: UIEdgeInsets,
    ) -> MKMapRect {
        // SAFETY: The iOS SDK declares `UIEdgeInsets` for this selector; the local type preserves
        // its `{UIEdgeInsets=CGFloat,CGFloat,CGFloat,CGFloat}` encoding.
        unsafe { self.mapRectThatFits_edgePadding(rect, padding) }
    }

    /// Retains and returns MapKit's current camera.
    pub fn camera(&self) -> Retained<MKMapCamera> {
        // SAFETY: The public `camera` property returns a retained MKMapCamera.
        unsafe { self.native_camera() }
    }

    /// Sets MapKit's camera.
    pub fn set_camera(&self, camera: &MKMapCamera, animated: bool) {
        // SAFETY: `camera` is a live public MKMapCamera; MapKit copies the property value.
        unsafe { self.setCamera_animated(camera, animated) };
    }

    /// Adds one native overlay at MapKit's default level.
    pub fn add_overlay(&self, overlay: &ProtocolObject<dyn MKOverlay>) {
        // SAFETY: MapKit accepts a live object conforming to its public MKOverlay protocol.
        unsafe { self.addOverlay(overlay) };
    }

    /// Adds one native overlay at the requested MapKit level.
    pub fn add_overlay_at_level(
        &self,
        overlay: &ProtocolObject<dyn MKOverlay>,
        level: MKOverlayLevel,
    ) {
        // SAFETY: The object and enum match the public `addOverlay:level:` signature.
        unsafe { self.addOverlay_level(overlay, level) };
    }

    /// Removes one native overlay without changing other overlays.
    pub fn remove_overlay(&self, overlay: &ProtocolObject<dyn MKOverlay>) {
        // SAFETY: MapKit accepts a live object conforming to its public MKOverlay protocol.
        unsafe { self.removeOverlay(overlay) };
    }

    /// Retains the renderer MapKit assigned to a native overlay, if it has one.
    pub fn renderer_for_overlay(
        &self,
        overlay: &ProtocolObject<dyn MKOverlay>,
    ) -> Option<Retained<MKOverlayRenderer>> {
        // SAFETY: The overlay is live and the public selector returns a nullable renderer object.
        unsafe { self.rendererForOverlay(overlay) }
    }

    /// Adds a native annotation.
    pub fn add_annotation(&self, annotation: &ProtocolObject<dyn MKAnnotation>) {
        // SAFETY: MapKit accepts a live object conforming to its public MKAnnotation protocol.
        unsafe { self.addAnnotation(annotation) };
    }

    /// Removes a native annotation.
    pub fn remove_annotation(&self, annotation: &ProtocolObject<dyn MKAnnotation>) {
        // SAFETY: MapKit accepts a live object conforming to its public MKAnnotation protocol.
        unsafe { self.removeAnnotation(annotation) };
    }

    /// Selects a native annotation and asks the delegate for its annotation view.
    pub fn select_annotation(&self, annotation: &ProtocolObject<dyn MKAnnotation>, animated: bool) {
        // SAFETY: The annotation conforms to MapKit's public protocol and the BOOL ABI is exact.
        unsafe { self.selectAnnotation_animated(annotation, animated) };
    }

    /// Deselects one native annotation, or all selected annotations when passed `None`.
    pub fn deselect_annotation(
        &self,
        annotation: Option<&ProtocolObject<dyn MKAnnotation>>,
        animated: bool,
    ) {
        // SAFETY: The optional object and BOOL match the public selector's nullable ABI.
        unsafe { self.deselectAnnotation_animated(annotation, animated) };
    }

    /// Retains and returns the current weak native delegate, if one is installed.
    pub fn delegate(&self) -> Option<Retained<ProtocolObject<dyn IosMapViewDelegate>>> {
        // SAFETY: The getter returns the weak Objective-C protocol object as an owned retain.
        unsafe { self.native_delegate() }
    }
}

#[allow(non_snake_case)]
impl NativeMapView {
    extern_methods!(
        // SAFETY: Matches the public iOS `MKMapView` declaration of the `UIView` initializer.
        #[unsafe(method(initWithFrame:))]
        #[unsafe(method_family = init)]
        pub(super) unsafe fn initWithFrame(this: Allocated<Self>, frame: CGRect) -> Retained<Self>;

        // SAFETY: Matches the public `MKCoordinateRegion` property getter in `MKMapView.h`.
        #[unsafe(method(region))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn native_region(&self) -> MKCoordinateRegion;

        // SAFETY: Matches the public `setRegion:animated:` declaration and by-value region ABI.
        #[unsafe(method(setRegion:animated:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn setRegion_animated(&self, region: MKCoordinateRegion, animated: bool);

        // SAFETY: Matches the public MKMapView projected visible-map-rectangle getter.
        #[unsafe(method(visibleMapRect))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn visibleMapRect(&self) -> MKMapRect;

        // SAFETY: Matches the public iOS by-value MKMapRect setter ABI.
        #[unsafe(method(setVisibleMapRect:animated:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn setVisibleMapRect_animated(&self, rect: MKMapRect, animated: bool);

        // SAFETY: Matches the public iOS by-value MKMapRect fitting ABI.
        #[unsafe(method(mapRectThatFits:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn mapRectThatFits(&self, rect: MKMapRect) -> MKMapRect;

        // SAFETY: The iOS SDK declares UIEdgeInsets, not NSEdgeInsets, for this public selector.
        #[unsafe(method(setVisibleMapRect:edgePadding:animated:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn setVisibleMapRect_edgePadding_animated(
            &self,
            rect: MKMapRect,
            padding: UIEdgeInsets,
            animated: bool,
        );

        // SAFETY: The iOS SDK declares UIEdgeInsets, not NSEdgeInsets, for this public selector.
        #[unsafe(method(mapRectThatFits:edgePadding:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn mapRectThatFits_edgePadding(
            &self,
            rect: MKMapRect,
            padding: UIEdgeInsets,
        ) -> MKMapRect;

        // SAFETY: Matches the public copied MKMapCamera property getter ABI.
        #[unsafe(method(camera))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn native_camera(&self) -> Retained<MKMapCamera>;

        // SAFETY: Matches the public setCamera:animated: selector and object argument ABI.
        #[unsafe(method(setCamera:animated:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn setCamera_animated(&self, camera: &MKMapCamera, animated: bool);

        // SAFETY: Matches the public iOS MKMapView `showsUserLocation` BOOL property setter ABI.
        #[unsafe(method(setShowsUserLocation:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn setShowsUserLocation(&self, shows_user_location: bool);

        // SAFETY: The generated public enum has the NSInteger encoding declared by the iOS SDK.
        #[unsafe(method(setUserTrackingMode:animated:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn setUserTrackingMode_animated(
            &self,
            mode: MKUserTrackingMode,
            animated: bool,
        );

        // SAFETY: Matches the public iOS MKMapView annotation API and its id<MKAnnotation> ABI.
        #[unsafe(method(addAnnotation:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn addAnnotation(&self, annotation: &ProtocolObject<dyn MKAnnotation>);

        // SAFETY: Matches the public iOS MKMapView annotation API and its id<MKAnnotation> ABI.
        #[unsafe(method(removeAnnotation:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn removeAnnotation(&self, annotation: &ProtocolObject<dyn MKAnnotation>);

        // SAFETY: Mirrors the public weak iOS MKMapViewDelegate property, whose runtime protocol
        // contains the locally declared exact renderer selector used by this crate.
        #[unsafe(method(delegate))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn native_delegate(
            &self,
        ) -> Option<Retained<ProtocolObject<dyn IosMapViewDelegate>>>;

        // SAFETY: Mirrors the public weak iOS MKMapViewDelegate property and its object ABI.
        #[unsafe(method(setDelegate:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn set_native_delegate(
            &self,
            delegate: Option<&ProtocolObject<dyn IosMapViewDelegate>>,
        );

        // SAFETY: Mirrors the public MKMapView overlay methods and id<MKOverlay> argument ABI.
        #[unsafe(method(addOverlay:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn addOverlay(&self, overlay: &ProtocolObject<dyn MKOverlay>);

        // SAFETY: Matches the public iOS addOverlay:level: selector and enum ABI.
        #[unsafe(method(addOverlay:level:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn addOverlay_level(
            &self,
            overlay: &ProtocolObject<dyn MKOverlay>,
            level: MKOverlayLevel,
        );

        // SAFETY: Mirrors the public MKMapView overlay methods and id<MKOverlay> argument ABI.
        #[unsafe(method(removeOverlay:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn removeOverlay(&self, overlay: &ProtocolObject<dyn MKOverlay>);

        // SAFETY: Matches the public nullable rendererForOverlay: selector and object ABI.
        #[unsafe(method(rendererForOverlay:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn rendererForOverlay(
            &self,
            overlay: &ProtocolObject<dyn MKOverlay>,
        ) -> Option<Retained<MKOverlayRenderer>>;

        // SAFETY: Matches the public annotation-selection selector and BOOL ABI.
        #[unsafe(method(selectAnnotation:animated:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn selectAnnotation_animated(
            &self,
            annotation: &ProtocolObject<dyn MKAnnotation>,
            animated: bool,
        );

        // SAFETY: Matches the public nullable annotation-deselection selector and BOOL ABI.
        #[unsafe(method(deselectAnnotation:animated:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn deselectAnnotation_animated(
            &self,
            annotation: Option<&ProtocolObject<dyn MKAnnotation>>,
            animated: bool,
        );
    );
}
