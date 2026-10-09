/// At least one returned call is outgoing.
pub const CALL_STATE_OUTGOING: u32 = 1 << 0;
/// At least one returned call is connected.
pub const CALL_STATE_CONNECTED: u32 = 1 << 1;
/// At least one returned call is on hold.
pub const CALL_STATE_ON_HOLD: u32 = 1 << 2;
/// At least one returned call has ended.
pub const CALL_STATE_ENDED: u32 = 1 << 3;

/// Rust-owned, point-in-time summary of the calls returned by CallKit.
///
/// The flags are an independent bitwise union across the returned calls. They do not describe a
/// single call or correlate one flag with another. The snapshot has no stable call identity and
/// may be stale as soon as the query returns.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[must_use]
pub struct CallActivitySnapshot {
    active_call_count: u64,
    state_flags: u32,
}

impl CallActivitySnapshot {
    pub(crate) const fn from_parts(active_call_count: u64, state_flags: u32) -> Self {
        Self {
            active_call_count,
            state_flags,
        }
    }

    /// Return the number of calls in the list returned by `CXCallObserver.calls`.
    pub const fn active_call_count(self) -> u64 {
        self.active_call_count
    }

    /// Return the ORed state flags for all calls in the list returned by CallKit.
    pub const fn state_flags(self) -> u32 {
        self.state_flags
    }

    /// Return whether any call is outgoing.
    pub const fn has_outgoing_call(self) -> bool {
        self.state_flags & CALL_STATE_OUTGOING != 0
    }

    /// Return whether any call is connected.
    pub const fn has_connected_call(self) -> bool {
        self.state_flags & CALL_STATE_CONNECTED != 0
    }

    /// Return whether any call is on hold.
    pub const fn has_call_on_hold(self) -> bool {
        self.state_flags & CALL_STATE_ON_HOLD != 0
    }

    /// Return whether any call has ended.
    pub const fn has_ended_call(self) -> bool {
        self.state_flags & CALL_STATE_ENDED != 0
    }
}
