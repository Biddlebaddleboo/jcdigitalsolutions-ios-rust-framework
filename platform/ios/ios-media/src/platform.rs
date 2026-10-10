use framework_media::MediaTime;
use objc2_core_media::{CMTime, CMTimeFlags};

const _: () = {
    assert!(core::mem::size_of::<CMTime>() == 24);
    assert!(core::mem::align_of::<CMTime>() == 4);
    assert!(core::mem::offset_of!(CMTime, value) == 0);
    assert!(core::mem::offset_of!(CMTime, timescale) == 8);
    assert!(core::mem::offset_of!(CMTime, flags) == 12);
    assert!(core::mem::offset_of!(CMTime, epoch) == 16);
};

/// A copyable CoreMedia time created from a portable finite `MediaTime`.
#[derive(Clone, Copy)]
pub struct IosMediaTime {
    value: CMTime,
}

impl IosMediaTime {
    /// Creates a CoreMedia time with epoch zero and a positive timescale.
    pub fn from_portable(time: MediaTime) -> Self {
        // The finite rational maps to an exact numeric value without calling CMTimeMake.
        let value = CMTime {
            value: time.value(),
            timescale: time.timescale(),
            flags: CMTimeFlags::Valid,
            epoch: 0,
        };
        Self { value }
    }

    /// Returns the native `CMTime` struct by value.
    pub const fn as_cm_time(self) -> CMTime {
        self.value
    }
}
