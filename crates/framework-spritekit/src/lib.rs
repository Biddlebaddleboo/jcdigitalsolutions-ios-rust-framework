#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "A small portable contract for one SpriteKit node position."]
#![doc = "\n\nThe contract does not model a scene, renderer, frame, action, or visual result."]

use framework_core::{Error, ErrorKind, Result};

/// A finite point in a SpriteKit node parent's coordinate system.
///
/// The coordinate unit is defined by the host scene. This value does not define pixels, view
/// coordinates, scaling, orientation, or a rendered location.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteNodePosition {
    x: f64,
    y: f64,
}

impl SpriteNodePosition {
    /// Creates a point when both coordinates are finite.
    pub fn new(x: f64, y: f64) -> Result<Self> {
        if !x.is_finite() || !y.is_finite() {
            return Err(Error::new(ErrorKind::InvalidInput));
        }
        Ok(Self { x, y })
    }

    /// Returns the horizontal coordinate.
    pub const fn x(self) -> f64 {
        self.x
    }

    /// Returns the vertical coordinate.
    pub const fn y(self) -> f64 {
        self.y
    }
}

/// A backend for reading and replacing one SpriteKit node's local position.
pub trait SpriteNodePositionBackend {
    /// Returns the node's position in its parent coordinate system.
    fn position(&self) -> Result<SpriteNodePosition>;

    /// Replaces the node's position after validating both coordinates.
    fn set_position(&mut self, position: SpriteNodePosition) -> Result<()>;
}
