use objc2::rc::{Allocated, Retained};
use objc2::{MainThreadMarker, MainThreadOnly, extern_class, extern_methods};
use objc2_core_foundation::CGRect;
use objc2_core_location::CLLocationCoordinate2D;
use objc2_map_kit::{MKCoordinateRegion, MKCoordinateSpan};
use objc2_ui_kit::UIView;

use crate::{MapCoordinate, MapRegion, MapSpan};

/// A retained native MapKit view bound to the main thread.
///
/// The type is a UIKit `UIView` subclass and can be embedded in a host controller's view
/// hierarchy. The host remains responsible for sizing and positioning it. MapKit may adjust a
/// requested region to fit the view's current dimensions; `region` reads back the native result.
pub struct IosMapView {
    native: Retained<NativeMapView>,
}

impl IosMapView {
    /// Creates an empty native `MKMapView` on the thread identified by `marker`.
    pub fn new(marker: MainThreadMarker) -> Self {
        // SAFETY: `MKMapView` publicly inherits `UIView` and declares `initWithFrame:`. The
        // zero-sized frame is a valid initial frame; the host assigns layout after embedding.
        let native =
            unsafe { NativeMapView::initWithFrame(NativeMapView::alloc(marker), CGRect::ZERO) };
        Self { native }
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
    /// thread.
    pub fn native_view(&self) -> &NativeMapView {
        &self.native
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
    );
}
