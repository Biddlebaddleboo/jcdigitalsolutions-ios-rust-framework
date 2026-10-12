use ios_map_view::{MapAnnotation, MapCoordinate};

#[test]
fn map_annotation_retains_caller_id_coordinate_and_title() {
    let coordinate = MapCoordinate::new(43.6532, -79.3832).unwrap();
    let marker = MapAnnotation::new(0x1234, coordinate, Some("pickup\0zone"));
    assert_eq!(marker.id(), 0x1234);
    assert_eq!(marker.coordinate(), coordinate);
    assert_eq!(marker.title(), Some("pickup\0zone"));
}

#[test]
fn map_annotation_supports_a_missing_title() {
    let coordinate = MapCoordinate::new(0.0, 0.0).unwrap();
    let marker = MapAnnotation::new(7, coordinate, None);
    assert_eq!(marker.title(), None);
}
