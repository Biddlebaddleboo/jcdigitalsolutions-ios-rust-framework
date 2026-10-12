#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Main-thread UIKit-hostable MapKit map view with validated regions and bounded overlays."]

pub use framework_maps::{MapAnnotation, MapCoordinate, MapRegion, MapSpan};

mod annotations;
pub use annotations::AnnotationSnapshotError;

mod delegate;
pub use delegate::MapDelegateError;

mod route;
pub use route::{MAX_ROUTE_COORDINATES, MapRouteError};

mod polygon;
pub use polygon::{MAX_POLYGON_COORDINATES, MapPolygonError};

mod tracking;
pub use tracking::{IosUserTrackingMode, MapUserTrackingError};

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{IosMapView, IosMapViewDelegate, NativeMapAnnotationView, NativeMapView};

#[cfg(target_os = "ios")]
pub use objc2_core_location::CLLocationCoordinate2D;

#[cfg(target_os = "ios")]
pub use objc2_map_kit::{
    MKAnnotation, MKCoordinateRegion, MKCoordinateSpan, MKMapCamera, MKMapPoint, MKMapRect,
    MKMapSize, MKOverlay, MKOverlayLevel, MKOverlayPathRenderer, MKOverlayRenderer,
    MKPointAnnotation, MKPolygon, MKPolygonRenderer, MKPolyline, MKPolylineRenderer,
    MKUserTrackingMode,
};

#[cfg(target_os = "ios")]
pub use objc2_ui_kit::{UIEdgeInsets, UIEvent, UIView};
