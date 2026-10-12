#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Portable, framework-owned map and spatial capability values."]

/// A system snapshot of support for an ARKit world-tracking configuration.
///
/// This value reports only the platform's configuration-support predicate. It does not mean an
/// AR session is active, camera access is authorized, or tracking will succeed in a particular
/// environment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorldTrackingSupport(bool);

impl WorldTrackingSupport {
    /// Creates a support value from a platform-reported predicate.
    pub const fn from_system(supported: bool) -> Self {
        Self(supported)
    }

    /// Returns whether the platform reports support for world tracking.
    pub const fn is_supported(self) -> bool {
        self.0
    }
}

/// A finite latitude/longitude pair in degrees accepted by MapKit's coordinate APIs.
///
/// Latitude must be in `-90..=90` and longitude in `-180..=180`. This type does not identify a
/// datum or imply that a coordinate came from the device or a location service.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapCoordinate {
    latitude_degrees: f64,
    longitude_degrees: f64,
}

impl MapCoordinate {
    /// Creates a coordinate when both values are finite and within their geographic degree ranges.
    pub fn new(latitude_degrees: f64, longitude_degrees: f64) -> Option<Self> {
        if !latitude_degrees.is_finite()
            || !longitude_degrees.is_finite()
            || !(-90.0..=90.0).contains(&latitude_degrees)
            || !(-180.0..=180.0).contains(&longitude_degrees)
        {
            return None;
        }

        Some(Self {
            latitude_degrees,
            longitude_degrees,
        })
    }

    /// Returns the latitude in degrees.
    pub const fn latitude_degrees(self) -> f64 {
        self.latitude_degrees
    }

    /// Returns the longitude in degrees.
    pub const fn longitude_degrees(self) -> f64 {
        self.longitude_degrees
    }
}

/// A finite latitude/longitude span in degrees accepted by a MapKit map region.
///
/// Latitude delta must be in `(0, 180]` and longitude delta in `(0, 360]`. These geographic
/// bounds reject empty, non-finite, and larger-than-world spans before they reach MapKit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapSpan {
    latitude_delta_degrees: f64,
    longitude_delta_degrees: f64,
}

impl MapSpan {
    /// Creates a span when both finite degree deltas are positive and within one world.
    pub fn new(latitude_delta_degrees: f64, longitude_delta_degrees: f64) -> Option<Self> {
        if !latitude_delta_degrees.is_finite()
            || !longitude_delta_degrees.is_finite()
            || latitude_delta_degrees <= 0.0
            || latitude_delta_degrees > 180.0
            || longitude_delta_degrees <= 0.0
            || longitude_delta_degrees > 360.0
        {
            return None;
        }

        Some(Self {
            latitude_delta_degrees,
            longitude_delta_degrees,
        })
    }

    /// Returns the north-to-south span in degrees.
    pub const fn latitude_delta_degrees(self) -> f64 {
        self.latitude_delta_degrees
    }

    /// Returns the east-to-west span in degrees.
    pub const fn longitude_delta_degrees(self) -> f64 {
        self.longitude_delta_degrees
    }
}

/// A validated center and geographic span for a native MapKit map view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapRegion {
    center: MapCoordinate,
    span: MapSpan,
}

impl MapRegion {
    /// Creates a region from a previously validated center and span.
    pub const fn new(center: MapCoordinate, span: MapSpan) -> Self {
        Self { center, span }
    }

    /// Returns the validated geographic center.
    pub const fn center(self) -> MapCoordinate {
        self.center
    }

    /// Returns the validated geographic span.
    pub const fn span(self) -> MapSpan {
        self.span
    }
}

/// A caller-owned map marker value with an identity stable across snapshots.
///
/// The identifier is chosen by the application and must be unique within each annotation
/// snapshot passed to an iOS map view. The optional title is copied by MapKit when applied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapAnnotation<'title> {
    id: u64,
    coordinate: MapCoordinate,
    title: Option<&'title str>,
}

impl<'title> MapAnnotation<'title> {
    /// Creates a marker value from an application-defined identifier, validated coordinate, and
    /// optional title.
    pub const fn new(id: u64, coordinate: MapCoordinate, title: Option<&'title str>) -> Self {
        Self {
            id,
            coordinate,
            title,
        }
    }

    /// Returns the caller-defined stable identifier.
    pub const fn id(self) -> u64 {
        self.id
    }

    /// Returns the validated marker coordinate.
    pub const fn coordinate(self) -> MapCoordinate {
        self.coordinate
    }

    /// Returns the optional marker title.
    pub const fn title(self) -> Option<&'title str> {
        self.title
    }
}

/// A finite point in MapKit's two-dimensional map projection.
///
/// The `x` and `y` units are MapKit map points, not meters or screen points. Finite values are
/// accepted without asserting that they lie within the world rectangle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapPoint {
    x: f64,
    y: f64,
}

impl MapPoint {
    /// Creates a map point when both projected coordinates are finite.
    pub fn new(x: f64, y: f64) -> Option<Self> {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }

        Some(Self { x, y })
    }

    /// Returns the point's `x` coordinate in map points.
    pub const fn x(self) -> f64 {
        self.x
    }

    /// Returns the point's `y` coordinate in map points.
    pub const fn y(self) -> f64 {
        self.y
    }
}

/// A finite, non-negative distance in meters reported by MapKit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapDistanceMeters(f64);

impl MapDistanceMeters {
    /// Creates a distance when the value is finite and non-negative.
    pub fn new(meters: f64) -> Option<Self> {
        if !meters.is_finite() || meters < 0.0 {
            return None;
        }

        Some(Self(meters))
    }

    /// Returns the distance in meters.
    pub const fn meters(self) -> f64 {
        self.0
    }
}
