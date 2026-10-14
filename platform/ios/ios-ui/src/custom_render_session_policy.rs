//! Deterministic lifecycle transitions for a custom-render UIKit session.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SessionPhase {
    Active,
    Suspended,
    Detached,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SessionEvent {
    Activate,
    Suspend,
    Detach,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SessionEffect {
    None,
    Activate,
    Suspend,
    Detach,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PresentDecision {
    Allowed,
    Suspended,
    Detached,
}

pub(crate) const fn present_decision(phase: SessionPhase) -> PresentDecision {
    match phase {
        SessionPhase::Active => PresentDecision::Allowed,
        SessionPhase::Suspended => PresentDecision::Suspended,
        SessionPhase::Detached => PresentDecision::Detached,
    }
}

pub(crate) const fn transition(
    phase: SessionPhase,
    event: SessionEvent,
) -> (SessionPhase, SessionEffect) {
    match (phase, event) {
        (SessionPhase::Active, SessionEvent::Suspend) => {
            (SessionPhase::Suspended, SessionEffect::Suspend)
        }
        (SessionPhase::Suspended, SessionEvent::Activate) => {
            (SessionPhase::Active, SessionEffect::Activate)
        }
        (SessionPhase::Active | SessionPhase::Suspended, SessionEvent::Detach) => {
            (SessionPhase::Detached, SessionEffect::Detach)
        }
        _ => (phase, SessionEffect::None),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PresentDecision, SessionEffect, SessionEvent, SessionPhase, present_decision, transition,
    };

    #[test]
    fn suspension_and_activation_are_idempotent_and_reversible() {
        assert_eq!(
            transition(SessionPhase::Active, SessionEvent::Suspend),
            (SessionPhase::Suspended, SessionEffect::Suspend)
        );
        assert_eq!(
            transition(SessionPhase::Suspended, SessionEvent::Suspend),
            (SessionPhase::Suspended, SessionEffect::None)
        );
        assert_eq!(
            transition(SessionPhase::Suspended, SessionEvent::Activate),
            (SessionPhase::Active, SessionEffect::Activate)
        );
        assert_eq!(
            transition(SessionPhase::Active, SessionEvent::Activate),
            (SessionPhase::Active, SessionEffect::None)
        );
    }

    #[test]
    fn detach_is_permanent_and_idempotent_from_each_live_phase() {
        for phase in [SessionPhase::Active, SessionPhase::Suspended] {
            assert_eq!(
                transition(phase, SessionEvent::Detach),
                (SessionPhase::Detached, SessionEffect::Detach)
            );
        }
        for event in [
            SessionEvent::Activate,
            SessionEvent::Suspend,
            SessionEvent::Detach,
        ] {
            assert_eq!(
                transition(SessionPhase::Detached, event),
                (SessionPhase::Detached, SessionEffect::None)
            );
        }
    }

    #[test]
    fn frame_submission_is_rejected_while_suspended_or_detached() {
        assert_eq!(
            present_decision(SessionPhase::Active),
            PresentDecision::Allowed
        );
        assert_eq!(
            present_decision(SessionPhase::Suspended),
            PresentDecision::Suspended
        );
        assert_eq!(
            present_decision(SessionPhase::Detached),
            PresentDecision::Detached
        );
    }
}
