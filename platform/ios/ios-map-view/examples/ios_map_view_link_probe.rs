#![deny(warnings)]

#[cfg(target_os = "ios")]
fn main() {
    use ios_map_view::{IosMapView, MapCoordinate, MapRegion, MapSpan};

    let Some(marker) = objc2::MainThreadMarker::new() else {
        return;
    };
    let view = IosMapView::new(marker);
    if let (Some(center), Some(span)) = (
        MapCoordinate::new(43.6532, -79.3832),
        MapSpan::new(0.25, 0.5),
    ) {
        view.set_region(MapRegion::new(center, span), false);
    }
    let _ = core::hint::black_box((view.region(), view.native_view()));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
