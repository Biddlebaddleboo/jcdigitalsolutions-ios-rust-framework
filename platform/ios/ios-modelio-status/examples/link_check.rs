#[cfg(target_os = "ios")]
fn main() {
    let _ = ios_modelio_status::can_import_file_extension("usdz");
}

#[cfg(not(target_os = "ios"))]
fn main() {}
