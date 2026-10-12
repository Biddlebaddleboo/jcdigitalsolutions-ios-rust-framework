#![cfg_attr(not(target_os = "ios"), no_std)]

use core::fmt;
use core::ops::BitOr;

#[cfg(target_os = "ios")]
mod platform;

/// Public AVAudioSession category identifiers supported by this iOS 10+ crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioSessionCategory {
    /// Background sounds that mix with other audio.
    Ambient,
    /// Foreground audio that silences other audio.
    SoloAmbient,
    /// Audio playback that is not silenced by the Ring/Silent switch.
    Playback,
    /// Audio recording without playback.
    Record,
    /// Simultaneous audio recording and playback.
    PlayAndRecord,
    /// Multiple distinct input and output routes.
    MultiRoute,
}

/// Public AVAudioSession modes available at the crate's iOS 10 deployment floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioSessionMode {
    /// The default mode.
    Default,
    /// A mode intended for voice communication with `PlayAndRecord`.
    VoiceChat,
    /// A mode intended for video communication with `PlayAndRecord`.
    VideoChat,
    /// A mode intended for video recording with `Record` or `PlayAndRecord`.
    VideoRecording,
    /// A mode that minimizes system processing.
    Measurement,
    /// A mode intended for movie playback with `Playback`.
    MoviePlayback,
    /// A mode intended for spoken-audio playback.
    SpokenAudio,
}

/// Category options for mixing and route behavior.
///
/// Apple validates whether each option is valid for the selected category and mode.
/// Invalid combinations are returned as [`IosAudioSessionError::NativeFailure`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AudioSessionCategoryOptions(u8);

impl AudioSessionCategoryOptions {
    const VALID_BITS: u8 = 0x7f;

    /// No category options.
    pub const NONE: Self = Self(0);
    /// Mix this app's audio with audio from other active apps.
    pub const MIX_WITH_OTHERS: Self = Self(1 << 0);
    /// Duck other audio while this app's session is active.
    pub const DUCK_OTHERS: Self = Self(1 << 1);
    /// Allow Bluetooth Hands-Free Profile routes for input.
    pub const ALLOW_BLUETOOTH_HFP: Self = Self(1 << 2);
    /// Prefer the built-in speaker over the receiver when no other route is connected.
    pub const DEFAULT_TO_SPEAKER: Self = Self(1 << 3);
    /// Interrupt other spoken-audio sessions while mixing with other audio.
    pub const INTERRUPT_SPOKEN_AUDIO_AND_MIX_WITH_OTHERS: Self = Self(1 << 4);
    /// Allow Bluetooth A2DP routes for output.
    pub const ALLOW_BLUETOOTH_A2DP: Self = Self(1 << 5);
    /// Allow AirPlay routes where supported.
    pub const ALLOW_AIRPLAY: Self = Self(1 << 6);

    /// Creates options from known bits, rejecting unknown values.
    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits & !Self::VALID_BITS == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    /// Returns the framework-owned option bits.
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Returns whether all bits in `other` are present.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Combines category options.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[cfg(any(target_os = "ios", test))]
    const fn native_bits(self) -> u32 {
        let mut bits = 0;
        if self.contains(Self::MIX_WITH_OTHERS) {
            bits |= 0x01;
        }
        if self.contains(Self::DUCK_OTHERS) {
            bits |= 0x02;
        }
        if self.contains(Self::ALLOW_BLUETOOTH_HFP) {
            bits |= 0x04;
        }
        if self.contains(Self::DEFAULT_TO_SPEAKER) {
            bits |= 0x08;
        }
        if self.contains(Self::INTERRUPT_SPOKEN_AUDIO_AND_MIX_WITH_OTHERS) {
            bits |= 0x11;
        }
        if self.contains(Self::ALLOW_BLUETOOTH_A2DP) {
            bits |= 0x20;
        }
        if self.contains(Self::ALLOW_AIRPLAY) {
            bits |= 0x40;
        }
        bits
    }
}

impl BitOr for AudioSessionCategoryOptions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

/// Options accepted when explicitly deactivating the shared session.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AudioSessionDeactivationOptions(u8);

impl AudioSessionDeactivationOptions {
    /// Deactivate without notifying other apps.
    pub const NONE: Self = Self(0);
    /// Ask the system to notify interrupted apps that they may resume playback.
    pub const NOTIFY_OTHERS: Self = Self(1);

    /// Returns whether the system should notify other apps on deactivation.
    pub const fn notify_others(self) -> bool {
        self.0 & Self::NOTIFY_OTHERS.0 != 0
    }
}

/// An explicit category, mode, and option selection for the process audio session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioSessionConfiguration {
    category: AudioSessionCategory,
    mode: AudioSessionMode,
    options: AudioSessionCategoryOptions,
}

impl AudioSessionConfiguration {
    /// Creates a configuration with no category options.
    pub const fn new(category: AudioSessionCategory, mode: AudioSessionMode) -> Self {
        Self {
            category,
            mode,
            options: AudioSessionCategoryOptions::NONE,
        }
    }

    /// Sets the exact category-option set to apply.
    pub const fn with_options(mut self, options: AudioSessionCategoryOptions) -> Self {
        self.options = options;
        self
    }

    /// Returns the selected category.
    pub const fn category(self) -> AudioSessionCategory {
        self.category
    }

    /// Returns the selected mode.
    pub const fn mode(self) -> AudioSessionMode {
        self.mode
    }

    /// Returns the selected options.
    pub const fn options(self) -> AudioSessionCategoryOptions {
        self.options
    }
}

/// The explicit shared audio-session operation that failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IosAudioSessionOperation {
    /// Category, mode, or options configuration.
    Configure,
    /// Session activation.
    Activate,
    /// Session deactivation.
    Deactivate,
}

/// Rust-owned failure from an explicit AVAudioSession call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IosAudioSessionError {
    /// The public AVFAudio call returned an NSError code.
    NativeFailure {
        /// The explicit operation that failed.
        operation: IosAudioSessionOperation,
        /// The native NSError code, copied into a Rust integer.
        code: i64,
    },
    /// A documented AVFAudio category or mode constant was unavailable at runtime.
    NativeConstantUnavailable {
        /// The operation requiring the unavailable constant.
        operation: IosAudioSessionOperation,
    },
}

impl fmt::Display for IosAudioSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NativeFailure { operation, code } => {
                write!(
                    f,
                    "AVAudioSession {operation:?} failed with native error {code}"
                )
            }
            Self::NativeConstantUnavailable { operation } => {
                write!(
                    f,
                    "AVAudioSession constant unavailable during {operation:?}"
                )
            }
        }
    }
}

/// A zero-state handle for the process-wide shared iOS audio session.
///
/// Creating or dropping this handle does not configure or activate the session.
pub struct IosAudioSession;

impl IosAudioSession {
    /// Creates an inert handle for the shared `AVAudioSession` singleton.
    pub const fn shared() -> Self {
        Self
    }
}

#[cfg(any(target_os = "ios", test))]
const fn native_failure(operation: IosAudioSessionOperation, code: i64) -> IosAudioSessionError {
    IosAudioSessionError::NativeFailure { operation, code }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_keeps_the_explicit_category_mode_and_options() {
        let options = AudioSessionCategoryOptions::ALLOW_BLUETOOTH_HFP
            | AudioSessionCategoryOptions::DEFAULT_TO_SPEAKER
            | AudioSessionCategoryOptions::ALLOW_BLUETOOTH_A2DP;
        let config = AudioSessionConfiguration::new(
            AudioSessionCategory::PlayAndRecord,
            AudioSessionMode::VoiceChat,
        )
        .with_options(options);
        assert_eq!(config.category(), AudioSessionCategory::PlayAndRecord);
        assert_eq!(config.mode(), AudioSessionMode::VoiceChat);
        assert_eq!(config.options().bits(), 0x2c);
        assert_eq!(config.options().native_bits(), 0x2c);
    }

    #[test]
    fn every_category_option_maps_to_its_public_apple_bits() {
        let cases = [
            (AudioSessionCategoryOptions::MIX_WITH_OTHERS, 0x01),
            (AudioSessionCategoryOptions::DUCK_OTHERS, 0x02),
            (AudioSessionCategoryOptions::ALLOW_BLUETOOTH_HFP, 0x04),
            (AudioSessionCategoryOptions::DEFAULT_TO_SPEAKER, 0x08),
            (
                AudioSessionCategoryOptions::INTERRUPT_SPOKEN_AUDIO_AND_MIX_WITH_OTHERS,
                0x11,
            ),
            (AudioSessionCategoryOptions::ALLOW_BLUETOOTH_A2DP, 0x20),
            (AudioSessionCategoryOptions::ALLOW_AIRPLAY, 0x40),
        ];
        for (option, expected) in cases {
            assert_eq!(
                option.native_bits(),
                expected,
                "option bits: {:#04x}",
                option.bits()
            );
        }
    }

    #[test]
    fn unknown_category_option_bits_are_rejected() {
        assert_eq!(
            AudioSessionCategoryOptions::from_bits(0x7f).unwrap().bits(),
            0x7f
        );
        assert_eq!(AudioSessionCategoryOptions::from_bits(0x80), None);
    }

    #[test]
    fn native_error_mapping_preserves_each_operation_and_signed_code() {
        for operation in [
            IosAudioSessionOperation::Configure,
            IosAudioSessionOperation::Activate,
            IosAudioSessionOperation::Deactivate,
        ] {
            assert_eq!(
                native_failure(operation, -66680),
                IosAudioSessionError::NativeFailure {
                    operation,
                    code: -66680,
                }
            );
        }
    }

    #[test]
    fn deactivation_notify_option_is_explicit() {
        assert!(!AudioSessionDeactivationOptions::NONE.notify_others());
        assert!(AudioSessionDeactivationOptions::NOTIFY_OTHERS.notify_others());
    }
}
