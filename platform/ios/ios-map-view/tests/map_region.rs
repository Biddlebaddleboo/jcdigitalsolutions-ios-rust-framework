use framework_maps::{MapCoordinate, MapRegion, MapSpan};

#[test]
fn map_span_accepts_world_boundaries_and_tiny_positive_values() {
    assert_eq!(
        MapSpan::new(180.0, 360.0).map(MapSpan::latitude_delta_degrees),
        Some(180.0)
    );
    assert_eq!(
        MapSpan::new(180.0, 360.0).map(MapSpan::longitude_delta_degrees),
        Some(360.0)
    );
    assert!(MapSpan::new(f64::MIN_POSITIVE, f64::MIN_POSITIVE).is_some());
}

#[test]
fn map_span_rejects_non_finite_empty_negative_and_over_world_values() {
    for latitude in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -0.1, 180.1] {
        assert!(MapSpan::new(latitude, 1.0).is_none(), "latitude {latitude}");
    }
    for longitude in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -0.1, 360.1] {
        assert!(
            MapSpan::new(1.0, longitude).is_none(),
            "longitude {longitude}"
        );
    }
}

#[test]
fn map_region_preserves_validated_center_and_span() {
    let center = MapCoordinate::new(43.6532, -79.3832).unwrap();
    let span = MapSpan::new(0.25, 0.5).unwrap();
    let region = MapRegion::new(center, span);
    assert_eq!(region.center(), center);
    assert_eq!(region.span(), span);
}
