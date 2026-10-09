#![cfg(target_os = "ios")]
#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![doc = "Read one documented extension-point identifier from a caller-supplied iOS `.appex` bundle."]
#![doc = "\n\nThis metadata snapshot does not load extension code or establish extension installation, approval, entitlement, launch, or runtime support."]

extern crate alloc;

mod platform;

pub use platform::{ExtensionMetadataError, read_extension_point_identifier};
