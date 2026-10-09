#[cfg(target_os = "ios")]
fn main() {
    let _ = ios_ui::text_metrics::system_font_metrics(12.0);
}

#[cfg(not(target_os = "ios"))]
fn main() {}
