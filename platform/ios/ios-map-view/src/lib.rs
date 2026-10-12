#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "Main-thread UIKit-hostable MapKit map view with validated geographic regions."]

pub use framework_maps::{MapCoordinate, MapRegion, MapSpan};

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::{IosMapView, NativeMapView};
