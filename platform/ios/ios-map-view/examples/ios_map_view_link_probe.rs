#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    use ios_map_view::{IosMapView, MapAnnotation, MapCoordinate, MapRegion, MapSpan};

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
    }
    let _ = core::hint::black_box((view.region(), view.native_view()));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
