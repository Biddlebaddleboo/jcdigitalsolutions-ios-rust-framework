#![cfg(target_os = "ios")]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "A bounded iOS adapter for a detached SpriteKit SKNode position."]
#![doc = "\n\nThis crate does not create a scene or view, attach a node, or render content."]

mod platform;

pub use platform::{IosSpriteNode, NativeSkNode};
