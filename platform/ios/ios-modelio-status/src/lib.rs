#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![doc = "An iOS-only ModelIO file-extension support query."]
#![doc = "\n\nThis crate asks whether ModelIO can read an asset file with a supplied extension. It does not parse, validate, or render an asset, and it makes no GPU support claim."]

#[cfg(target_os = "ios")]
mod platform;

#[cfg(target_os = "ios")]
pub use platform::can_import_file_extension;
