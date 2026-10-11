use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::{CallActionCompletionError, CallEndReason, CallId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CallPhase {
    Reporting,
    Active,
    Ending(CallEndReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BeginReportError {
    Duplicate,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EndTransition {
    ReportNow(CallEndReason),
    Deferred,
    AlreadyEnding,
    NotFound,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReportTransition {
    Accepted,
    EndAfterAccept(CallEndReason),
    Rejected,
    Stale,
}

#[derive(Default)]
pub(crate) struct CallRegistry {
    calls: HashMap<CallId, (u64, CallPhase)>,
    generation: u64,
    invalidated: bool,
}

/// One-shot handoff for a native completion that may arrive on any queue.
pub(crate) struct OnceSlot<T> {
    value: Mutex<Option<T>>,
}

const ACTION_PENDING: u8 = 0;
const ACTION_QUEUED: u8 = 1;
const ACTION_RESOLVING: u8 = 2;
const ACTION_RESOLVED: u8 = 3;
const ACTION_EXPIRED: u8 = 4;
const ACTION_UNAVAILABLE: u8 = 5;

/// Thread-safe one-shot gate; native action access remains in the main-thread registry.
#[derive(Default)]
pub(crate) struct DeferredActionGate(AtomicU8);

impl DeferredActionGate {
    pub(crate) fn submit(&self) -> Result<(), CallActionCompletionError> {
        match self.0.compare_exchange(
            ACTION_PENDING,
            ACTION_QUEUED,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => Ok(()),
            Err(ACTION_EXPIRED) => Err(CallActionCompletionError::Expired),
            Err(ACTION_UNAVAILABLE) => Err(CallActionCompletionError::ProviderUnavailable),
            Err(_) => Err(CallActionCompletionError::AlreadyCompleted),
        }
    }

    pub(crate) fn begin_resolution(&self) -> bool {
        self.0
            .compare_exchange(
                ACTION_QUEUED,
                ACTION_RESOLVING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }

    pub(crate) fn resolve(&self) -> bool {
        self.0
            .compare_exchange(
                ACTION_RESOLVING,
                ACTION_RESOLVED,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }

    pub(crate) fn expire(&self) {
        self.terminate(ACTION_EXPIRED);
    }

    pub(crate) fn invalidate_unavailable(&self) {
        self.terminate(ACTION_UNAVAILABLE);
    }

    fn terminate(&self, terminal: u8) {
        let mut current = self.0.load(Ordering::Acquire);
        while matches!(current, ACTION_PENDING | ACTION_QUEUED | ACTION_RESOLVING) {
            match self.0.compare_exchange_weak(
                current,
                terminal,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                Err(next) => current = next,
            }
        }
    }
}

impl<T> OnceSlot<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            value: Mutex::new(Some(value)),
        }
    }

    pub(crate) fn take(&self) -> Option<T> {
        self.value
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
}

impl CallRegistry {
    pub(crate) fn begin_report(&mut self, id: CallId) -> Result<u64, BeginReportError> {
        if self.invalidated {
            return Err(BeginReportError::Unavailable);
        }
        if self.calls.contains_key(&id) {
            return Err(BeginReportError::Duplicate);
        }
        self.calls
            .insert(id, (self.generation, CallPhase::Reporting));
        Ok(self.generation)
    }

    pub(crate) fn report_completed(
        &mut self,
        id: CallId,
        generation: u64,
        accepted: bool,
    ) -> ReportTransition {
        if self.invalidated || generation != self.generation {
            return ReportTransition::Stale;
        }
        let Some((call_generation, phase)) = self.calls.remove(&id) else {
            return ReportTransition::Stale;
        };
        if call_generation != generation {
            self.calls.insert(id, (call_generation, phase));
            return ReportTransition::Stale;
        }
        if !accepted {
            return ReportTransition::Rejected;
        }
        match phase {
            CallPhase::Reporting => {
                self.calls.insert(id, (generation, CallPhase::Active));
                ReportTransition::Accepted
            }
            CallPhase::Ending(reason) => ReportTransition::EndAfterAccept(reason),
            CallPhase::Active => {
                self.calls.insert(id, (generation, CallPhase::Active));
                ReportTransition::Stale
            }
        }
    }

    pub(crate) fn end(&mut self, id: CallId, reason: CallEndReason) -> EndTransition {
        if self.invalidated {
            return EndTransition::Unavailable;
        }
        let Some((generation, phase)) = self.calls.remove(&id) else {
            return EndTransition::NotFound;
        };
        match phase {
            CallPhase::Reporting => {
                self.calls
                    .insert(id, (generation, CallPhase::Ending(reason)));
                EndTransition::Deferred
            }
            CallPhase::Active => EndTransition::ReportNow(reason),
            CallPhase::Ending(existing_reason) => {
                self.calls
                    .insert(id, (generation, CallPhase::Ending(existing_reason)));
                EndTransition::AlreadyEnding
            }
        }
    }

    pub(crate) fn is_active(&self, id: CallId) -> bool {
        !self.invalidated
            && self
                .calls
                .get(&id)
                .is_some_and(|(_, phase)| *phase == CallPhase::Active)
    }

    pub(crate) fn finish_end_action(&mut self, id: CallId) -> bool {
        if self.is_active(id) {
            self.calls.remove(&id);
            true
        } else {
            false
        }
    }

    pub(crate) fn reset(&mut self) {
        self.calls.clear();
        self.advance_generation();
    }

    pub(crate) fn invalidate(&mut self) {
        if self.invalidated {
            return;
        }
        self.calls.clear();
        self.advance_generation();
        self.invalidated = true;
    }

    fn advance_generation(&mut self) {
        if let Some(next) = self.generation.checked_add(1) {
            self.generation = next;
        } else {
            self.invalidated = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    fn id(value: u8) -> CallId {
        CallId::from_bytes([value; 16])
    }

    #[test]
    fn report_acceptance_activates_unique_call() {
        let mut registry = CallRegistry::default();
        let generation = registry.begin_report(id(1)).unwrap();
        assert_eq!(
            registry.begin_report(id(1)),
            Err(BeginReportError::Duplicate)
        );
        assert_eq!(
            registry.report_completed(id(1), generation, true),
            ReportTransition::Accepted
        );
        assert!(registry.is_active(id(1)));
        assert_eq!(
            registry.report_completed(id(1), generation, true),
            ReportTransition::Stale
        );
        assert!(registry.is_active(id(1)));
    }

    #[test]
    fn rejected_and_failed_reports_remove_the_call() {
        let mut registry = CallRegistry::default();
        let generation = registry.begin_report(id(2)).unwrap();
        assert_eq!(
            registry.report_completed(id(2), generation, false),
            ReportTransition::Rejected
        );
        assert_eq!(
            registry.end(id(2), CallEndReason::Failed),
            EndTransition::NotFound
        );
    }

    #[test]
    fn end_during_report_is_sent_only_after_acceptance() {
        let mut registry = CallRegistry::default();
        let generation = registry.begin_report(id(3)).unwrap();
        assert_eq!(
            registry.end(id(3), CallEndReason::RemoteEnded),
            EndTransition::Deferred
        );
        assert_eq!(
            registry.report_completed(id(3), generation, true),
            ReportTransition::EndAfterAccept(CallEndReason::RemoteEnded)
        );
        assert_eq!(
            registry.end(id(3), CallEndReason::RemoteEnded),
            EndTransition::NotFound
        );
    }

    #[test]
    fn failed_report_after_deferred_end_does_not_send_end() {
        let mut registry = CallRegistry::default();
        let generation = registry.begin_report(id(4)).unwrap();
        assert_eq!(
            registry.end(id(4), CallEndReason::Unanswered),
            EndTransition::Deferred
        );
        assert_eq!(
            registry.report_completed(id(4), generation, false),
            ReportTransition::Rejected
        );
    }

    #[test]
    fn answer_requires_active_call_and_end_action_removes_it_once() {
        let mut registry = CallRegistry::default();
        let generation = registry.begin_report(id(5)).unwrap();
        assert!(!registry.is_active(id(5)));
        registry.report_completed(id(5), generation, true);
        assert!(registry.is_active(id(5)));
        assert!(registry.finish_end_action(id(5)));
        assert!(!registry.finish_end_action(id(5)));
    }

    #[test]
    fn reset_invalidates_late_completion_and_permits_new_calls() {
        let mut registry = CallRegistry::default();
        let old_generation = registry.begin_report(id(6)).unwrap();
        registry.reset();
        assert_eq!(
            registry.report_completed(id(6), old_generation, true),
            ReportTransition::Stale
        );
        assert!(registry.begin_report(id(6)).is_ok());
    }

    #[test]
    fn invalidate_clears_calls_and_rejects_future_reports() {
        let mut registry = CallRegistry::default();
        registry.begin_report(id(7)).unwrap();
        registry.invalidate();
        assert_eq!(
            registry.begin_report(id(8)),
            Err(BeginReportError::Unavailable)
        );
        assert_eq!(
            registry.end(id(7), CallEndReason::Failed),
            EndTransition::Unavailable
        );
    }

    #[test]
    fn generation_overflow_fails_closed() {
        let mut registry = CallRegistry {
            generation: u64::MAX,
            ..CallRegistry::default()
        };
        registry.reset();
        assert_eq!(
            registry.begin_report(id(9)),
            Err(BeginReportError::Unavailable)
        );
    }

    #[test]
    fn repeated_invalidation_does_not_reopen_or_advance_state() {
        let mut registry = CallRegistry::default();
        registry.invalidate();
        let generation = registry.generation;
        registry.invalidate();
        assert_eq!(registry.generation, generation);
        assert_eq!(
            registry.begin_report(id(10)),
            Err(BeginReportError::Unavailable)
        );
    }

    #[test]
    fn report_completion_context_is_claimed_once_across_queues() {
        let context = Arc::new(OnceSlot::new(17_u8));
        let barrier = Arc::new(Barrier::new(3));
        let threads = (0..2)
            .map(|_| {
                let context = Arc::clone(&context);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    context.take().is_some()
                })
            })
            .collect::<Vec<_>>();
        barrier.wait();
        assert_eq!(
            threads
                .into_iter()
                .map(|thread| thread.join().expect("callback thread must finish"))
                .filter(|claimed| *claimed)
                .count(),
            1
        );
        assert_eq!(context.take(), None);
    }

    #[test]
    fn deferred_action_has_one_queued_resolution() {
        let gate = DeferredActionGate::default();
        assert_eq!(gate.submit(), Ok(()));
        assert_eq!(
            gate.submit(),
            Err(CallActionCompletionError::AlreadyCompleted)
        );
        assert!(gate.begin_resolution());
        assert!(gate.resolve());
        assert!(!gate.begin_resolution());
        assert_eq!(
            gate.submit(),
            Err(CallActionCompletionError::AlreadyCompleted)
        );
    }

    #[test]
    fn timeout_rejects_pending_and_queued_late_results() {
        let pending = DeferredActionGate::default();
        pending.expire();
        assert_eq!(pending.submit(), Err(CallActionCompletionError::Expired));

        let queued = DeferredActionGate::default();
        assert_eq!(queued.submit(), Ok(()));
        queued.expire();
        assert!(!queued.begin_resolution());
        assert_eq!(queued.submit(), Err(CallActionCompletionError::Expired));
    }

    #[test]
    fn provider_reset_rejects_pending_late_results() {
        let gate = DeferredActionGate::default();
        gate.invalidate_unavailable();
        assert_eq!(
            gate.submit(),
            Err(CallActionCompletionError::ProviderUnavailable)
        );
    }

    #[test]
    fn provider_invalidation_rejects_queued_late_results() {
        let gate = DeferredActionGate::default();
        assert_eq!(gate.submit(), Ok(()));
        gate.invalidate_unavailable();
        assert!(!gate.begin_resolution());
        assert_eq!(
            gate.submit(),
            Err(CallActionCompletionError::ProviderUnavailable)
        );
    }

    #[test]
    fn call_end_expires_queued_action_before_resolution() {
        let mut calls = CallRegistry::default();
        let generation = calls.begin_report(id(11)).unwrap();
        assert_eq!(
            calls.report_completed(id(11), generation, true),
            ReportTransition::Accepted
        );
        let gate = DeferredActionGate::default();
        assert_eq!(gate.submit(), Ok(()));
        assert!(calls.finish_end_action(id(11)));
        gate.expire();
        assert!(!gate.begin_resolution());
        assert_eq!(gate.submit(), Err(CallActionCompletionError::Expired));
        assert!(!calls.finish_end_action(id(11)));
    }

    #[test]
    fn provider_drop_invalidates_a_queued_action() {
        let gate = DeferredActionGate::default();
        assert_eq!(gate.submit(), Ok(()));
        gate.invalidate_unavailable();
        assert!(!gate.begin_resolution());
        assert_eq!(
            gate.submit(),
            Err(CallActionCompletionError::ProviderUnavailable)
        );
    }

    #[test]
    fn concurrent_deferred_results_claim_one_resolution() {
        let gate = Arc::new(DeferredActionGate::default());
        let barrier = Arc::new(Barrier::new(3));
        let threads = (0..2)
            .map(|_| {
                let gate = Arc::clone(&gate);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    gate.submit()
                })
            })
            .collect::<Vec<_>>();
        barrier.wait();
        let results = threads
            .into_iter()
            .map(|thread| thread.join().expect("completion thread must finish"))
            .collect::<Vec<_>>();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| { **result == Err(CallActionCompletionError::AlreadyCompleted) })
                .count(),
            1
        );
    }
}
