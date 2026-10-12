#![cfg(target_os = "ios")]
#![deny(unsafe_op_in_unsafe_fn)]

//! Small iOS-only adapters for UIKit thread proof and foreign-callback panic containment.
//!
//! This crate does not own a process-wide runtime, executor, object registry, or UI model.
//! Objective-C objects stay under Objective-C ownership, and UIKit work stays on the main
//! thread. It intentionally exposes no portable API and does not wrap capability services.

#[cfg(feature = "dispatch")]
pub mod dispatch;
#[cfg(feature = "dispatch")]
mod dispatch_gate;
pub mod ffi;
pub mod main_thread;
#[cfg(feature = "dispatch-timer")]
pub mod repeating_timer;
#[cfg(feature = "dispatch-timer")]
mod timer_policy;
