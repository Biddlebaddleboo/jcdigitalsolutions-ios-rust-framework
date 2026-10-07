#![cfg_attr(target_os = "ios", deny(unsafe_op_in_unsafe_fn))]

#[cfg(target_os = "ios")]
mod app;

#[cfg(target_os = "ios")]
fn main() {
    app::run();
}

#[cfg(not(target_os = "ios"))]
fn main() {
    eprintln!("Build this example for an iOS target; see README.md");
}
