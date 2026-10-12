#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    use ios_map_view::{
        IosMapView, IosUserTrackingMode, MapAnnotation, MapCoordinate, MapRegion, MapSpan,
        MapUserTrackingError,
    };

    let Some(marker) = objc2::MainThreadMarker::new() else {
        return;
    };
    let mut view = IosMapView::new(marker);
    if let (Some(center), Some(span)) = (
        MapCoordinate::new(43.6532, -79.3832),
        MapSpan::new(0.25, 0.5),
    ) {
        view.set_region(MapRegion::new(center, span), false);
        view.replace_annotations(&[MapAnnotation::new(1, center, Some("probe marker"))])
            .expect("unique marker id");
        view.replace_annotations(&[])
            .expect("empty marker snapshot");
        let route = [
            MapCoordinate::new(43.6532, -79.3832).unwrap(),
            MapCoordinate::new(43.6540, -79.3820).unwrap(),
        ];
        view.replace_route(&route).expect("valid route");
        let polygon = [
            MapCoordinate::new(43.6532, -79.3832).unwrap(),
            MapCoordinate::new(43.6540, -79.3820).unwrap(),
            MapCoordinate::new(43.6525, -79.3810).unwrap(),
        ];
        view.replace_polygon(&polygon).expect("valid polygon");
        view.clear_route();
        view.clear_polygon();
    }
    let tracking_api: fn(
        &IosMapView,
        IosUserTrackingMode,
        bool,
    ) -> Result<(), MapUserTrackingError> = IosMapView::set_user_tracking_mode;
    let _ = core::hint::black_box((view.region(), view.native_view(), tracking_api));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
