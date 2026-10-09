#![deny(warnings)]

#[cfg(target_os = "ios")]
use framework_data::ByteView;
#[cfg(target_os = "ios")]
use ios_data::IosData;

#[cfg(target_os = "ios")]
fn main() {
    let source = [0_u8, 1, 2, 255];
    if let Ok(data) = IosData::copy_from(ByteView::new(&source)) {
        let _ = core::hint::black_box(data.copy_to_owned());
        let _ = core::hint::black_box(data.as_cf_data());
    }
}

#[cfg(not(target_os = "ios"))]
fn main() {}
