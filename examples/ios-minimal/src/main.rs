#![cfg_attr(target_os = "ios", deny(unsafe_op_in_unsafe_fn))]

#[cfg(all(
    target_os = "ios",
    any(feature = "legacy-app-demo", feature = "custom-render-metal-demo")
))]
mod app;

#[cfg(all(
    target_os = "ios",
    not(any(feature = "legacy-app-demo", feature = "custom-render-metal-demo"))
))]
mod scene_app;

#[cfg(target_os = "ios")]
fn main() {
    #[cfg(any(feature = "legacy-app-demo", feature = "custom-render-metal-demo"))]
    app::run();
    #[cfg(not(any(feature = "legacy-app-demo", feature = "custom-render-metal-demo")))]
    scene_app::run();
}

#[cfg(not(target_os = "ios"))]
fn main() {
    eprintln!("Build this example for an iOS target; see README.md");
}
