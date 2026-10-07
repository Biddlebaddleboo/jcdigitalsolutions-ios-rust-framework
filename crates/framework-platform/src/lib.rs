#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Zero-sized target markers and compile-time backend association traits."]

use framework_core::Platform;

mod sealed {
    pub trait Sealed {}
}

/// A zero-sized compile-time target marker.
pub trait PlatformMarker: sealed::Sealed + Copy + 'static {
    /// The semantic platform represented by this marker.
    const PLATFORM: Platform;
}

/// Marker for Apple iOS backends.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Ios;
impl sealed::Sealed for Ios {}
impl PlatformMarker for Ios {
    const PLATFORM: Platform = Platform::Ios;
}

/// Marker for Android backends.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Android;
impl sealed::Sealed for Android {}
impl PlatformMarker for Android {
    const PLATFORM: Platform = Platform::Android;
}

/// Marker for Apple macOS backends.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct MacOS;
impl sealed::Sealed for MacOS {}
impl PlatformMarker for MacOS {
    const PLATFORM: Platform = Platform::MacOS;
}

/// Marker for Microsoft Windows backends.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Windows;
impl sealed::Sealed for Windows {}
impl PlatformMarker for Windows {
    const PLATFORM: Platform = Platform::Windows;
}

/// Marker for Linux backends.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Linux;
impl sealed::Sealed for Linux {}
impl PlatformMarker for Linux {
    const PLATFORM: Platform = Platform::Linux;
}

/// Marker for browser-facing WebAssembly backends.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Web;
impl sealed::Sealed for Web {}
impl PlatformMarker for Web {
    const PLATFORM: Platform = Platform::Web;
}

/// Marker used when a target has no dedicated framework marker.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Unknown;
impl sealed::Sealed for Unknown {}
impl PlatformMarker for Unknown {
    const PLATFORM: Platform = Platform::Unknown;
}

/// The marker selected from the compilation target by `cfg`, without runtime lookup.
#[cfg(target_os = "ios")]
pub type CurrentPlatform = Ios;
/// The marker selected from the compilation target by `cfg`, without runtime lookup.
#[cfg(target_os = "android")]
pub type CurrentPlatform = Android;
/// The marker selected from the compilation target by `cfg`, without runtime lookup.
#[cfg(target_os = "macos")]
pub type CurrentPlatform = MacOS;
/// The marker selected from the compilation target by `cfg`, without runtime lookup.
#[cfg(target_os = "windows")]
pub type CurrentPlatform = Windows;
/// The marker selected from the compilation target by `cfg`, without runtime lookup.
#[cfg(target_os = "linux")]
pub type CurrentPlatform = Linux;
/// The marker selected from the compilation target by `cfg`, without runtime lookup.
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub type CurrentPlatform = Web;
/// The marker selected from the compilation target by `cfg`, without runtime lookup.
#[cfg(not(any(
    target_os = "ios",
    target_os = "android",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    all(target_arch = "wasm32", target_os = "unknown")
)))]
pub type CurrentPlatform = Unknown;

/// Associates a backend implementation type with one statically selected platform marker.
pub trait PlatformBackend {
    /// The target marker that selects this backend at compile time.
    type Target: PlatformMarker;
}

/// Reports whether a marker is the marker selected for this build target.
pub const fn is_current<P: PlatformMarker>() -> bool {
    P::PLATFORM as u8 == Platform::current() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{align_of, size_of};

    #[test]
    fn platform_markers_are_zero_sized_and_static() {
        assert_eq!(size_of::<Ios>(), 0);
        assert_eq!(align_of::<Ios>(), 1);
        assert_eq!(size_of::<CurrentPlatform>(), 0);
        assert_eq!(
            <CurrentPlatform as PlatformMarker>::PLATFORM,
            Platform::current()
        );
        assert!(is_current::<CurrentPlatform>());
        assert_eq!(
            is_current::<Android>(),
            Platform::current() == Platform::Android
        );
    }

    struct IosBackend;
    impl PlatformBackend for IosBackend {
        type Target = Ios;
    }

    #[test]
    fn backend_association_is_a_type_level_marker() {
        assert_eq!(
            <IosBackend as PlatformBackend>::Target::PLATFORM,
            Platform::Ios
        );
    }
}
