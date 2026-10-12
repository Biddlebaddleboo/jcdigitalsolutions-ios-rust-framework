/// The native map delegate slot could not accept a framework-owned delegate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MapDelegateError {
    /// A different delegate owns the native weak delegate slot, or a built-in overlay renderer is active.
    DelegateSlotOccupied,
}

#[cfg(any(target_os = "ios", test))]
pub(crate) const fn delegate_slot_can_be_claimed(
    is_empty: bool,
    is_owned: bool,
    has_builtin_overlays: bool,
) -> bool {
    !has_builtin_overlays && (is_empty || is_owned)
}

#[cfg(any(target_os = "ios", test))]
pub(crate) const fn delegate_slot_can_be_cleared(is_owned: bool) -> bool {
    is_owned
}

#[cfg(test)]
mod tests {
    use super::{delegate_slot_can_be_claimed, delegate_slot_can_be_cleared};

    #[test]
    fn delegate_claim_rejects_an_unowned_host_delegate() {
        assert!(!delegate_slot_can_be_claimed(false, false, false));
    }

    #[test]
    fn delegate_claim_accepts_an_empty_or_self_owned_slot() {
        assert!(delegate_slot_can_be_claimed(true, false, false));
        assert!(delegate_slot_can_be_claimed(false, true, false));
    }

    #[test]
    fn delegate_claim_rejects_active_builtin_overlays() {
        assert!(!delegate_slot_can_be_claimed(true, false, true));
    }

    #[test]
    fn cleanup_does_not_clear_a_delegate_replaced_by_the_host() {
        assert!(!delegate_slot_can_be_cleared(false));
        assert!(delegate_slot_can_be_cleared(true));
    }
}
