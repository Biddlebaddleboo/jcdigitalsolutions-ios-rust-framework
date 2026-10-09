#[cfg(target_os = "ios")]
use core::mem::{align_of, offset_of, size_of};

#[cfg(target_os = "ios")]
use framework_ui::Frame;
#[cfg(target_os = "ios")]
use objc2_core_foundation::{CGFloat, CGPoint, CGRect, CGSize};

#[cfg(target_os = "ios")]
const _: () = {
    assert!(size_of::<CGFloat>() == 8);
    assert!(align_of::<CGFloat>() == 8);
    assert!(size_of::<CGPoint>() == 16);
    assert!(align_of::<CGPoint>() == 8);
    assert!(offset_of!(CGPoint, x) == 0);
    assert!(offset_of!(CGPoint, y) == 8);
    assert!(size_of::<CGSize>() == 16);
    assert!(align_of::<CGSize>() == 8);
    assert!(offset_of!(CGSize, width) == 0);
    assert!(offset_of!(CGSize, height) == 8);
    assert!(size_of::<CGRect>() == 32);
    assert!(align_of::<CGRect>() == 8);
    assert!(offset_of!(CGRect, origin) == 0);
    assert!(offset_of!(CGRect, size) == 16);
};

#[cfg(target_os = "ios")]
fn main() {
    let first = Frame::new(0.0, 0.0, 16.0, 16.0).expect("valid first frame");
    let overlap = Frame::new(4.0, 5.0, 12.0, 12.0).expect("valid overlap frame");
    let disjoint = Frame::new(32.0, 32.0, 4.0, 4.0).expect("valid disjoint frame");
    let overlapping = ios_ui::geometry::intersection(first, overlap).expect("native intersection");
    let separate = ios_ui::geometry::intersection(first, disjoint).expect("native empty result");
    core::hint::black_box((overlapping, separate));
}

#[cfg(not(target_os = "ios"))]
fn main() {}
