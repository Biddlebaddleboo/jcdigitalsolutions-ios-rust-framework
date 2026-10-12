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
    MKAnnotation, MKCoordinateRegion, MKCoordinateSpan, MKOverlay, MKOverlayRenderer,
    MKPointAnnotation, MKPolyline, MKPolylineRenderer, MKUserTrackingMode,
};
use objc2_ui_kit::{UIColor, UIView};
use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::annotations::sorted_snapshot;
use crate::route::validate_route_point_count;
use crate::tracking::{
    IosUserTrackingMode, current_location_authorization, validate_user_tracking_request,
};
use crate::{
    AnnotationSnapshotError, MapAnnotation, MapCoordinate, MapRegion, MapRouteError, MapSpan,
    MapUserTrackingError,
};

struct AnnotationEntry {
    id: u64,
    native: Retained<MKPointAnnotation>,
}

struct RouteDelegateState {
    route: RefCell<Option<Retained<MKPolyline>>>,
}

extern_protocol!(
    /// The iOS route-render callback subset of Apple's public map-view delegate protocol.
    ///
    /// # Safety
    ///
    /// The protocol name, selector, object arguments, non-null return contract, and main-thread
    /// callback behavior match the public iOS `MKMapView.h` declaration.
    #[allow(clippy::missing_safety_doc)]
    #[name = "MKMapViewDelegate"]
    unsafe trait IosMapViewDelegate: NSObjectProtocol + MainThreadOnly {
        #[optional]
        #[unsafe(method(mapView:rendererForOverlay:))]
        #[unsafe(method_family = none)]
        unsafe fn map_view_renderer_for_overlay(
            &self,
            map_view: &NativeMapView,
            overlay: &ProtocolObject<dyn MKOverlay>,
        ) -> Retained<MKOverlayRenderer>;
    }
);

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - The route state is used only by MapKit's main-thread delegate callback.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "JcdIosMapRouteDelegate"]
    #[ivars = RouteDelegateState]
    struct RouteDelegate;

    // SAFETY: NSObject supplies all required NSObjectProtocol methods.
    unsafe impl NSObjectProtocol for RouteDelegate {}

    // SAFETY: This class conforms to Apple's public MKMapViewDelegate protocol and implements
    // its iOS rendererForOverlay selector with the SDK's exact Objective-C ABI.
    unsafe impl IosMapViewDelegate for RouteDelegate {
        #[unsafe(method_id(mapView:rendererForOverlay:))]
        unsafe fn map_view_renderer_for_overlay(
            &self,
            _map_view: &NativeMapView,
            overlay: &ProtocolObject<dyn MKOverlay>,
        ) -> Retained<MKOverlayRenderer> {
            let _keep_alive = self.retain();
            catch_unwind(AssertUnwindSafe(|| self.renderer_for_overlay(overlay)))
                .unwrap_or_else(|_| empty_overlay_renderer(overlay))
        }
    }
);

impl RouteDelegate {
    fn new(marker: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(marker).set_ivars(RouteDelegateState {
            route: RefCell::new(None),
        });
        // SAFETY: This is a fresh NSObject subclass allocation and NSObject's init is valid.
        unsafe { msg_send![super(this), init] }
    }

    fn replace_route(&self, route: Option<Retained<MKPolyline>>) -> Option<Retained<MKPolyline>> {
        core::mem::replace(&mut *self.ivars().route.borrow_mut(), route)
    }

    fn renderer_for_overlay(
        &self,
        overlay: &ProtocolObject<dyn MKOverlay>,
    ) -> Retained<MKOverlayRenderer> {
        let route = self.ivars().route.borrow().clone();
        let Some(route) = route else {
            return empty_overlay_renderer(overlay);
        };
        if !overlay_is_route(overlay, &route) {
            return empty_overlay_renderer(overlay);
        }

        // SAFETY: `MKPolylineRenderer` publicly accepts the exact retained `MKPolyline` held by
        // this delegate. MapKit calls this callback on the map's main thread.
        let renderer =
            unsafe { MKPolylineRenderer::initWithPolyline(MKPolylineRenderer::alloc(), &route) };
        let color = UIColor::blueColor();
        // SAFETY: The iOS SDK declares `setStrokeColor:` on MKOverlayPathRenderer with a UIColor
        // object parameter. The local objc2-map-kit binding omits this iOS-only setter.
        let () = unsafe { msg_send![&*renderer, setStrokeColor: &*color] };
        // SAFETY: The generated public binding matches MKOverlayPathRenderer's CGFloat setter.
        unsafe { renderer.setLineWidth(5.0) };
        // SAFETY: MKPolylineRenderer publicly subclasses MKOverlayRenderer.
        unsafe { Retained::cast_unchecked(renderer) }
    }
}

fn overlay_is_route(overlay: &ProtocolObject<dyn MKOverlay>, route: &MKPolyline) -> bool {
    let overlay_pointer = overlay as *const ProtocolObject<dyn MKOverlay> as *const ();
    let route_pointer = route as *const MKPolyline as *const ();
    ptr::eq(overlay_pointer, route_pointer)
}

fn empty_overlay_renderer(overlay: &ProtocolObject<dyn MKOverlay>) -> Retained<MKOverlayRenderer> {
    // SAFETY: `overlay` is a live public MKOverlay object supplied by MapKit.
    unsafe { MKOverlayRenderer::initWithOverlay(MKOverlayRenderer::alloc(), overlay) }
}

fn delegate_is_owned(
    current: &ProtocolObject<dyn IosMapViewDelegate>,
    owned: &RouteDelegate,
) -> bool {
    let current_pointer = current as *const ProtocolObject<dyn IosMapViewDelegate> as *const ();
    let owned_pointer = owned as *const RouteDelegate as *const ();
    ptr::eq(current_pointer, owned_pointer)
}

/// A retained native MapKit view bound to the main thread.
///
/// The type is a UIKit `UIView` subclass and can be embedded in a host controller's view
/// hierarchy. The host remains responsible for sizing and positioning it. MapKit may adjust a
/// requested region to fit the view's current dimensions; `region` reads back the native result.
pub struct IosMapView {
    native: Retained<NativeMapView>,
    annotations: Vec<AnnotationEntry>,
    route_delegate: Option<Retained<RouteDelegate>>,
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
            route_delegate: None,
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

        let current_delegate = unsafe { self.native.delegate() };
        let owns_current = current_delegate.as_ref().is_some_and(|current| {
            self.route_delegate
                .as_ref()
                .is_some_and(|owned| delegate_is_owned(current, owned))
        });
        if !crate::route::route_delegate_slot_is_available(current_delegate.is_none(), owns_current)
        {
            return Err(MapRouteError::DelegateSlotOccupied);
        }

        let mut native_coordinates: Vec<_> = coordinates
            .iter()
            .map(|coordinate| CLLocationCoordinate2D {
                latitude: coordinate.latitude_degrees(),
                longitude: coordinate.longitude_degrees(),
            })
            .collect();
        let point_count = native_coordinates.len() as NSUInteger;
        let points = NonNull::new(native_coordinates.as_mut_ptr())
            .expect("validated route contains at least two coordinates");
        // SAFETY: The bounded, non-empty coordinate buffer matches CLLocationCoordinate2D and
        // MapKit copies the coordinates while creating its owned MKPolyline.
        let route = unsafe { MKPolyline::polylineWithCoordinates_count(points, point_count) };

        let delegate = self
            .route_delegate
            .as_ref()
            .cloned()
            .unwrap_or_else(|| RouteDelegate::new(self.marker));
        if current_delegate.is_none() {
            // SAFETY: The map view is main-thread-bound and this wrapper retains the weak delegate.
            unsafe {
                self.native
                    .setDelegate(Some(ProtocolObject::from_ref(&*delegate)))
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
        self.route_delegate = Some(delegate);
        Ok(())
    }

    /// Removes the route path without changing marker annotations.
    pub fn clear_route(&mut self) {
        let Some(delegate) = self.route_delegate.take() else {
            return;
        };
        if let Some(route) = delegate.replace_route(None) {
            let route: &ProtocolObject<dyn MKOverlay> = ProtocolObject::from_ref(&*route);
            // SAFETY: Remove only the route overlay owned by this wrapper, on the map's main thread.
            unsafe { self.native.removeOverlay(route) };
        }
        let current_delegate = unsafe { self.native.delegate() };
        if current_delegate
            .as_ref()
            .is_some_and(|current| delegate_is_owned(current, &delegate))
        {
            // SAFETY: Clear only this wrapper's weak delegate slot, on the map's main thread.
            unsafe { self.native.setDelegate(None) };
        }
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
    /// map's route delegate.
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
        let region = unsafe { self.native.region() };
        let center = MapCoordinate::new(region.center.latitude, region.center.longitude)?;
        let span = MapSpan::new(region.span.latitudeDelta, region.span.longitudeDelta)?;
        Some(MapRegion::new(center, span))
    }

    /// Borrows the native `MKMapView` for embedding or other UIKit integration on the main
    /// thread. While a route is active, do not replace its delegate slot through this escape
    /// handle; route replacement reports `MapRouteError::DelegateSlotOccupied` if that slot is
    /// changed by the host.
    pub fn native_view(&self) -> &NativeMapView {
        &self.native
    }
}

impl Drop for IosMapView {
    fn drop(&mut self) {
        self.clear_route();
    }
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
        pub(super) unsafe fn region(&self) -> MKCoordinateRegion;

        // SAFETY: Matches the public `setRegion:animated:` declaration and by-value region ABI.
        #[unsafe(method(setRegion:animated:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn setRegion_animated(&self, region: MKCoordinateRegion, animated: bool);

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
        pub(super) unsafe fn delegate(
            &self,
        ) -> Option<Retained<ProtocolObject<dyn IosMapViewDelegate>>>;

        // SAFETY: Mirrors the public weak iOS MKMapViewDelegate property and its object ABI.
        #[unsafe(method(setDelegate:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn setDelegate(
            &self,
            delegate: Option<&ProtocolObject<dyn IosMapViewDelegate>>,
        );

        // SAFETY: Mirrors the public MKMapView overlay methods and id<MKOverlay> argument ABI.
        #[unsafe(method(addOverlay:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn addOverlay(&self, overlay: &ProtocolObject<dyn MKOverlay>);

        // SAFETY: Mirrors the public MKMapView overlay methods and id<MKOverlay> argument ABI.
        #[unsafe(method(removeOverlay:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn removeOverlay(&self, overlay: &ProtocolObject<dyn MKOverlay>);
    );
}
