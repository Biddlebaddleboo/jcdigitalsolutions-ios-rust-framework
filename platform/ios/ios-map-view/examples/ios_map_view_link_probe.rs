#![deny(warnings)]

#[cfg(target_os = "ios")]
mod probe {
    use core::ptr::NonNull;
    use ios_map_view::{
        IosMapView, IosMapViewDelegate, IosUserTrackingMode, MKAnnotation, MKMapPoint, MKMapRect,
        MKMapSize, MKOverlay, MKOverlayLevel, MKOverlayRenderer, MKPointAnnotation, MKPolyline,
        MapAnnotation, MapCoordinate, MapRegion, MapSpan, MapUserTrackingError,
        NativeMapAnnotationView, NativeMapView,
    };
    use objc2::define_class;
    use objc2::msg_send;
    use objc2::rc::Retained;
    use objc2::runtime::{NSObject, NSObjectProtocol, ProtocolObject};
    use objc2::{AnyThread, MainThreadMarker, MainThreadOnly};
    use objc2_core_foundation::{CGPoint, CGRect};
    use objc2_core_location::CLLocationCoordinate2D;
    use objc2_foundation::NSUInteger;
    use objc2_map_kit::MKPolylineRenderer;
    use objc2_ui_kit::UIView;

    define_class!(
        // SAFETY: NSObject has no subclassing requirements; all callbacks use MapKit's main thread.
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "JcdIosMapViewProbeDelegate"]
        #[ivars = ()]
        struct ProbeDelegate;

        // SAFETY: NSObject supplies all required NSObjectProtocol methods.
        unsafe impl NSObjectProtocol for ProbeDelegate {}

        // SAFETY: Each selector and argument matches the public iOS MKMapViewDelegate ABI.
        unsafe impl IosMapViewDelegate for ProbeDelegate {
            #[unsafe(method(mapView:regionWillChangeAnimated:))]
            unsafe fn map_view_region_will_change_animated(
                &self,
                _map_view: &NativeMapView,
                _animated: bool,
            ) {
            }

            #[unsafe(method(mapView:regionDidChangeAnimated:))]
            unsafe fn map_view_region_did_change_animated(
                &self,
                _map_view: &NativeMapView,
                _animated: bool,
            ) {
            }

            #[unsafe(method(mapViewDidChangeVisibleRegion:))]
            unsafe fn map_view_did_change_visible_region(&self, _map_view: &NativeMapView) {}

            #[unsafe(method(mapView:didSelectAnnotationView:))]
            unsafe fn map_view_did_select_annotation_view(
                &self,
                _map_view: &NativeMapView,
                _view: &NativeMapAnnotationView,
            ) {
            }

            #[unsafe(method(mapView:didDeselectAnnotationView:))]
            unsafe fn map_view_did_deselect_annotation_view(
                &self,
                _map_view: &NativeMapView,
                _view: &NativeMapAnnotationView,
            ) {
            }

            #[unsafe(method_id(mapView:rendererForOverlay:))]
            unsafe fn map_view_renderer_for_overlay(
                &self,
                _map_view: &NativeMapView,
                _overlay: &ProtocolObject<dyn MKOverlay>,
            ) -> Option<Retained<MKOverlayRenderer>> {
                None
            }
        }
    );

    impl ProbeDelegate {
        fn new(marker: MainThreadMarker) -> Retained<Self> {
            let this = Self::alloc(marker).set_ivars(());
            // SAFETY: This is a fresh NSObject subclass allocation.
            unsafe { msg_send![super(this), init] }
        }
    }

    pub fn run(marker: MainThreadMarker) {
        let mut view = IosMapView::new(marker);
        let Some(center) = MapCoordinate::new(43.6532, -79.3832) else {
            return;
        };
        let Some(span) = MapSpan::new(0.25, 0.5) else {
            return;
        };
        view.set_region(MapRegion::new(center, span), false);
        view.replace_annotations(&[MapAnnotation::new(1, center, Some("probe marker"))])
            .expect("unique marker id");
        view.replace_annotations(&[])
            .expect("empty marker snapshot");

        let route = [
            MapCoordinate::new(43.6532, -79.3832).expect("valid coordinate"),
            MapCoordinate::new(43.6540, -79.3820).expect("valid coordinate"),
        ];
        view.replace_route(&route).expect("valid route");
        let polygon = [
            MapCoordinate::new(43.6532, -79.3832).expect("valid coordinate"),
            MapCoordinate::new(43.6540, -79.3820).expect("valid coordinate"),
            MapCoordinate::new(43.6525, -79.3810).expect("valid coordinate"),
        ];
        view.replace_polygon(&polygon).expect("valid polygon");
        view.clear_route();
        view.clear_polygon();

        let map = view.native_view();
        let rect = MKMapRect {
            origin: MKMapPoint { x: 0.0, y: 0.0 },
            size: MKMapSize {
                width: 1.0,
                height: 1.0,
            },
        };
        map.set_visible_map_rect(rect, false);
        let fitted = map.map_rect_that_fits(rect);
        let padding = ios_map_view::UIEdgeInsets {
            top: 2.0,
            left: 3.0,
            bottom: 4.0,
            right: 5.0,
        };
        map.set_visible_map_rect_with_edge_padding(fitted, padding, false);
        let _ = map.map_rect_that_fits_with_edge_padding(fitted, padding);
        let camera = map.camera();
        map.set_camera(&camera, false);
        map.set_region(map.region(), false);
        let _ = map.visible_map_rect();

        let native_annotation = unsafe { MKPointAnnotation::new() };
        let annotation: &ProtocolObject<dyn MKAnnotation> =
            ProtocolObject::from_ref(&*native_annotation);
        map.add_annotation(annotation);
        map.select_annotation(annotation, false);
        map.deselect_annotation(Some(annotation), false);
        map.remove_annotation(annotation);

        let mut first_coordinates = [
            CLLocationCoordinate2D {
                latitude: 43.6532,
                longitude: -79.3832,
            },
            CLLocationCoordinate2D {
                latitude: 43.6540,
                longitude: -79.3820,
            },
        ];
        let mut second_coordinates = [
            CLLocationCoordinate2D {
                latitude: 43.6525,
                longitude: -79.3810,
            },
            CLLocationCoordinate2D {
                latitude: 43.6510,
                longitude: -79.3800,
            },
        ];
        let first = unsafe {
            MKPolyline::polylineWithCoordinates_count(
                NonNull::new(first_coordinates.as_mut_ptr()).expect("nonempty coordinates"),
                first_coordinates.len() as NSUInteger,
            )
        };
        let second = unsafe {
            MKPolyline::polylineWithCoordinates_count(
                NonNull::new(second_coordinates.as_mut_ptr()).expect("nonempty coordinates"),
                second_coordinates.len() as NSUInteger,
            )
        };
        let first_overlay: &ProtocolObject<dyn MKOverlay> = ProtocolObject::from_ref(&*first);
        let second_overlay: &ProtocolObject<dyn MKOverlay> = ProtocolObject::from_ref(&*second);
        map.add_overlay_at_level(first_overlay, MKOverlayLevel::AboveRoads);
        map.add_overlay_at_level(second_overlay, MKOverlayLevel::AboveLabels);
        let renderer =
            unsafe { MKPolylineRenderer::initWithPolyline(MKPolylineRenderer::alloc(), &first) };
        unsafe {
            renderer.setLineWidth(4.0);
            renderer.setLineDashPattern(None);
        }
        map.remove_overlay(first_overlay);
        map.remove_overlay(second_overlay);

        view.set_delegate(ProbeDelegate::new(marker))
            .expect("empty delegate slot");
        let _ = view.native_view().delegate();
        view.clear_delegate();

        let parent = UIView::new(marker);
        parent.addSubview(view.ui_view());
        let native_view = view.ui_view();
        native_view.setFrame(CGRect::ZERO);
        native_view.setBounds(CGRect::ZERO);
        native_view.setHidden(false);
        native_view.setNeedsLayout();
        native_view.layoutIfNeeded();
        let _ = native_view.hitTest_withEvent(CGPoint::ZERO, None);
        let _ = native_view.superview();
        native_view.removeFromSuperview();

        let tracking_api: fn(
            &IosMapView,
            IosUserTrackingMode,
            bool,
        ) -> Result<(), MapUserTrackingError> = IosMapView::set_user_tracking_mode;
        let _ = core::hint::black_box((view.region(), view.native_view(), tracking_api));
    }
}

#[cfg(target_os = "ios")]
fn main() {
    if let Some(marker) = objc2::MainThreadMarker::new() {
        probe::run(marker);
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}
