#![cfg(target_os = "ios")]
#![deny(unsafe_op_in_unsafe_fn)]

//! Small iOS-only adapters for UIKit thread proof and foreign-callback panic containment.
//!
//! This crate does not own a process-wide runtime, executor, object registry, or UI model.
//! Objective-C objects stay under Objective-C ownership, and UIKit work stays on the main
//! thread. It intentionally exposes no portable API and does not wrap capability services.

pub mod ffi;
pub mod main_thread;
