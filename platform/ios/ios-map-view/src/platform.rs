use core::mem;
use objc2::rc::{Allocated, Retained};
use objc2::runtime::ProtocolObject;
use objc2::{MainThreadMarker, MainThreadOnly, extern_class, extern_methods};
use objc2_core_foundation::CGRect;
use objc2_core_location::CLLocationCoordinate2D;
use objc2_foundation::NSString;
use objc2_map_kit::{MKAnnotation, MKCoordinateRegion, MKCoordinateSpan, MKPointAnnotation};
use objc2_ui_kit::UIView;

use crate::annotations::sorted_snapshot;
use crate::{AnnotationSnapshotError, MapAnnotation, MapCoordinate, MapRegion, MapSpan};

struct AnnotationEntry {
    id: u64,
    native: Retained<MKPointAnnotation>,
}

/// A retained native MapKit view bound to the main thread.
///
/// The type is a UIKit `UIView` subclass and can be embedded in a host controller's view
/// hierarchy. The host remains responsible for sizing and positioning it. MapKit may adjust a
/// requested region to fit the view's current dimensions; `region` reads back the native result.
pub struct IosMapView {
    native: Retained<NativeMapView>,
    annotations: Vec<AnnotationEntry>,
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
    /// thread.
    pub fn native_view(&self) -> &NativeMapView {
        &self.native
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

        // SAFETY: Matches the public iOS MKMapView annotation API and its id<MKAnnotation> ABI.
        #[unsafe(method(addAnnotation:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn addAnnotation(&self, annotation: &ProtocolObject<dyn MKAnnotation>);

        // SAFETY: Matches the public iOS MKMapView annotation API and its id<MKAnnotation> ABI.
        #[unsafe(method(removeAnnotation:))]
        #[unsafe(method_family = none)]
        pub(super) unsafe fn removeAnnotation(&self, annotation: &ProtocolObject<dyn MKAnnotation>);
    );
}
