//! Bounded, host-testable state for native touch event delivery.

use std::collections::VecDeque;

pub(crate) const MAX_ACTIVE_TOUCHES: usize = 64;
pub(crate) const MAX_QUEUED_TOUCH_EVENTS: usize = 1024;

/// UIKit phase for one contact sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IosTouchPhase {
    /// A contact began.
    Began,
    /// A contact moved or supplied a coalesced sample.
    Moved,
    /// A contact ended normally.
    Ended,
    /// UIKit or the adapter cancelled a contact.
    Cancelled,
}

/// UIKit-reported class of touch or pointer input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IosTouchKind {
    /// Direct finger contact.
    Direct,
    /// Indirect touch input.
    Indirect,
    /// Apple Pencil or equivalent stylus input.
    Pencil,
    /// Pointer input delivered as an indirect pointer.
    IndirectPointer,
    /// A future UIKit touch type, preserved as its raw integer value.
    Other(i64),
}

/// One immutable sample delivered by UIKit or synthesized for cancellation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IosTouchEvent {
    /// Stable nonzero identity for this contact sequence.
    pub id: u64,
    /// Contact lifecycle phase.
    pub phase: IosTouchPhase,
    /// UIKit monotonic timestamp in seconds.
    pub timestamp_seconds: f64,
    /// Horizontal coordinate in host-local UIKit points.
    pub x_points: f64,
    /// Vertical coordinate in host-local UIKit points.
    pub y_points: f64,
    /// Horizontal coordinate in host-local pixels, computed as points × host scale.
    pub x_pixels: f64,
    /// Vertical coordinate in host-local pixels, computed as points × host scale.
    pub y_pixels: f64,
    /// Device pixels per UIKit point at sample time.
    pub scale: f64,
    /// UIKit-reported contact source.
    pub kind: IosTouchKind,
    /// UIKit-reported tap count for the contact.
    pub tap_count: u32,
    /// UIKit pointer button bit mask, or zero when no event supplied one.
    pub button_mask: u64,
    /// Whether UIKit supplied this sample in its coalesced-touch array.
    pub coalesced: bool,
}

/// Sticky reason why the adapter stopped delivering new contact sequences.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IosTouchInputFault {
    /// The bounded queue could not preserve another sample and all active contacts were cancelled.
    EventCapacityExceeded,
    /// More simultaneous contacts arrived than the fixed contact table can hold.
    ContactCapacityExceeded,
    /// UIKit supplied a non-finite coordinate, timestamp, or invalid scale.
    InvalidSample,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TouchSample {
    pub native_key: usize,
    pub timestamp_seconds: f64,
    pub x_points: f64,
    pub y_points: f64,
    pub scale: f64,
    pub kind: IosTouchKind,
    pub tap_count: u32,
    pub button_mask: u64,
    pub coalesced: bool,
}

impl TouchSample {
    #[cfg(target_os = "ios")]
    pub(crate) const EMPTY: Self = Self {
        native_key: 0,
        timestamp_seconds: 0.0,
        x_points: 0.0,
        y_points: 0.0,
        scale: 0.0,
        kind: IosTouchKind::Direct,
        tap_count: 0,
        button_mask: 0,
        coalesced: false,
    };

    fn event(self, id: u64, phase: IosTouchPhase) -> IosTouchEvent {
        IosTouchEvent {
            id,
            phase,
            timestamp_seconds: self.timestamp_seconds,
            x_points: self.x_points,
            y_points: self.y_points,
            x_pixels: self.x_points * self.scale,
            y_pixels: self.y_points * self.scale,
            scale: self.scale,
            kind: self.kind,
            tap_count: self.tap_count,
            button_mask: self.button_mask,
            coalesced: self.coalesced,
        }
    }

    fn is_valid(self) -> bool {
        self.timestamp_seconds.is_finite()
            && self.timestamp_seconds >= 0.0
            && self.x_points.is_finite()
            && self.y_points.is_finite()
            && self.scale.is_finite()
            && self.scale > 0.0
            && (self.x_points * self.scale).is_finite()
            && (self.y_points * self.scale).is_finite()
    }
}

#[derive(Clone, Copy)]
struct ActiveContact {
    native_key: usize,
    id: u64,
    last_event: IosTouchEvent,
}

pub(crate) struct TouchQueue {
    contacts: Vec<ActiveContact>,
    events: VecDeque<IosTouchEvent>,
    next_id: u64,
    fault: Option<IosTouchInputFault>,
    disabled: bool,
}

impl TouchQueue {
    pub(crate) fn new() -> Self {
        Self {
            contacts: Vec::with_capacity(MAX_ACTIVE_TOUCHES),
            events: VecDeque::with_capacity(MAX_QUEUED_TOUCH_EVENTS),
            next_id: 1,
            fault: None,
            disabled: false,
        }
    }

    pub(crate) fn begin_batch(&mut self, samples: &mut [TouchSample]) {
        if self.disabled {
            return;
        }
        samples.sort_by(|left, right| {
            left.y_points
                .total_cmp(&right.y_points)
                .then_with(|| left.x_points.total_cmp(&right.x_points))
                .then_with(|| left.native_key.cmp(&right.native_key))
        });
        for sample in samples.iter().copied() {
            self.begin(sample);
        }
    }

    pub(crate) fn begin(&mut self, sample: TouchSample) {
        if self.disabled {
            return;
        }
        if !sample.is_valid() {
            self.trip(IosTouchInputFault::InvalidSample, sample.timestamp_seconds);
            return;
        }
        if self
            .contacts
            .iter()
            .any(|contact| contact.native_key == sample.native_key)
        {
            return;
        }
        if self.contacts.len() == MAX_ACTIVE_TOUCHES {
            self.trip(
                IosTouchInputFault::ContactCapacityExceeded,
                sample.timestamp_seconds,
            );
            return;
        }
        if self.events.len() + self.contacts.len() + 2 > MAX_QUEUED_TOUCH_EVENTS {
            self.trip(
                IosTouchInputFault::EventCapacityExceeded,
                sample.timestamp_seconds,
            );
            return;
        }
        let id = self.allocate_id();
        let event = sample.event(id, IosTouchPhase::Began);
        self.events.push_back(event);
        self.contacts.push(ActiveContact {
            native_key: sample.native_key,
            id,
            last_event: event,
        });
    }

    pub(crate) fn move_sample(&mut self, sample: TouchSample) {
        if self.disabled {
            return;
        }
        if !sample.is_valid() {
            self.trip(IosTouchInputFault::InvalidSample, sample.timestamp_seconds);
            return;
        }
        let Some(index) = self
            .contacts
            .iter()
            .position(|contact| contact.native_key == sample.native_key)
        else {
            return;
        };
        if self.events.len() + self.contacts.len() + 1 > MAX_QUEUED_TOUCH_EVENTS {
            self.trip(
                IosTouchInputFault::EventCapacityExceeded,
                sample.timestamp_seconds,
            );
            return;
        }
        let event = sample.event(self.contacts[index].id, IosTouchPhase::Moved);
        self.events.push_back(event);
        self.contacts[index].last_event = event;
    }

    pub(crate) fn end(&mut self, sample: TouchSample, phase: IosTouchPhase) {
        if self.disabled {
            return;
        }
        if !sample.is_valid() {
            self.trip(IosTouchInputFault::InvalidSample, sample.timestamp_seconds);
            return;
        }
        let Some(index) = self
            .contacts
            .iter()
            .position(|contact| contact.native_key == sample.native_key)
        else {
            return;
        };
        if self.events.len() + self.contacts.len() > MAX_QUEUED_TOUCH_EVENTS {
            self.trip(
                IosTouchInputFault::EventCapacityExceeded,
                sample.timestamp_seconds,
            );
            return;
        }
        let contact = self.contacts.remove(index);
        let event = sample.event(contact.id, phase);
        self.events.push_back(event);
    }

    pub(crate) fn cancel_all(&mut self) {
        if self.contacts.is_empty() {
            return;
        }
        let needed = self.contacts.len();
        if self.events.len() + needed > MAX_QUEUED_TOUCH_EVENTS {
            self.trip(
                IosTouchInputFault::EventCapacityExceeded,
                self.contacts[0].last_event.timestamp_seconds,
            );
            return;
        }
        for contact in self.contacts.drain(..) {
            let mut event = contact.last_event;
            event.phase = IosTouchPhase::Cancelled;
            event.coalesced = false;
            self.events.push_back(event);
        }
    }

    pub(crate) fn disable(&mut self) {
        self.cancel_all();
        self.disabled = true;
    }

    #[cfg(target_os = "ios")]
    pub(crate) fn reject_contact_overflow(&mut self, timestamp_seconds: f64) {
        if !self.disabled {
            self.trip(
                IosTouchInputFault::ContactCapacityExceeded,
                timestamp_seconds,
            );
        }
    }

    pub(crate) fn active_id(&self, native_key: usize) -> Option<u64> {
        self.contacts
            .iter()
            .find(|contact| contact.native_key == native_key)
            .map(|contact| contact.id)
    }

    pub(crate) fn pop_event(&mut self) -> Option<IosTouchEvent> {
        self.events.pop_front()
    }

    pub(crate) const fn fault(&self) -> Option<IosTouchInputFault> {
        self.fault
    }

    fn allocate_id(&mut self) -> u64 {
        loop {
            let id = self.next_id.max(1);
            self.next_id = id.wrapping_add(1).max(1);
            if !self.contacts.iter().any(|contact| contact.id == id) {
                return id;
            }
        }
    }

    fn trip(&mut self, fault: IosTouchInputFault, timestamp_seconds: f64) {
        if self.fault.is_none() {
            self.fault = Some(fault);
        }
        self.events.clear();
        for contact in self.contacts.drain(..) {
            let mut event = contact.last_event;
            event.phase = IosTouchPhase::Cancelled;
            event.timestamp_seconds = if timestamp_seconds.is_finite() && timestamp_seconds >= 0.0 {
                timestamp_seconds
            } else {
                event.timestamp_seconds
            };
            event.coalesced = false;
            self.events.push_back(event);
        }
        self.disabled = true;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        IosTouchInputFault, IosTouchKind, IosTouchPhase, MAX_ACTIVE_TOUCHES,
        MAX_QUEUED_TOUCH_EVENTS, TouchQueue, TouchSample,
    };

    fn sample(
        native_key: usize,
        x_points: f64,
        y_points: f64,
        timestamp_seconds: f64,
    ) -> TouchSample {
        TouchSample {
            native_key,
            timestamp_seconds,
            x_points,
            y_points,
            scale: 3.0,
            kind: IosTouchKind::Direct,
            tap_count: 1,
            button_mask: 0,
            coalesced: false,
        }
    }

    #[test]
    fn simultaneous_contacts_sort_began_and_retain_identity_across_reordered_moves() {
        let mut queue = TouchQueue::new();
        let mut began = [
            sample(20, 4.0, 8.0, 1.0),
            sample(10, 12.0, 2.0, 1.0),
            sample(30, 5.0, 2.0, 1.0),
        ];
        queue.begin_batch(&mut began);
        let first = queue.pop_event().expect("first began");
        let second = queue.pop_event().expect("second began");
        let third = queue.pop_event().expect("third began");
        assert_eq!((first.id, first.x_points), (1, 5.0));
        assert_eq!((second.id, second.x_points), (2, 12.0));
        assert_eq!((third.id, third.x_points), (3, 4.0));
        assert_eq!(first.phase, IosTouchPhase::Began);
        assert_eq!((first.x_pixels, first.y_pixels), (15.0, 6.0));

        queue.move_sample(sample(20, 7.0, 9.0, 1.1));
        queue.move_sample(sample(30, 6.0, 3.0, 1.2));
        assert_eq!(queue.pop_event().expect("first move").id, 3);
        assert_eq!(queue.pop_event().expect("second move").id, 1);
        queue.end(sample(10, 13.0, 3.0, 1.3), IosTouchPhase::Ended);
        let ended = queue.pop_event().expect("ended");
        assert_eq!(ended.id, 2);
        assert_eq!(ended.phase, IosTouchPhase::Ended);
        assert_eq!(queue.active_id(10), None);
        queue.move_sample(sample(10, 14.0, 4.0, 1.4));
        assert_eq!(queue.pop_event(), None, "stale moves are ignored");
    }

    #[test]
    fn coalesced_samples_preserve_native_timestamps_and_order() {
        let mut queue = TouchQueue::new();
        queue.begin(sample(1, 0.0, 0.0, 2.0));
        let _began = queue.pop_event();
        let mut older = sample(1, 1.0, 2.0, 2.1);
        older.coalesced = true;
        let mut newer = sample(1, 3.0, 4.0, 2.2);
        newer.coalesced = true;
        queue.move_sample(older);
        queue.move_sample(newer);
        let first = queue.pop_event().expect("older coalesced sample");
        let second = queue.pop_event().expect("newer coalesced sample");
        assert_eq!(first.timestamp_seconds, 2.1);
        assert_eq!(second.timestamp_seconds, 2.2);
        assert!(first.coalesced && second.coalesced);
    }

    #[test]
    fn native_and_teardown_cancellation_close_all_contacts_with_last_positions() {
        let mut queue = TouchQueue::new();
        let mut began = [sample(1, 1.0, 1.0, 3.0), sample(2, 2.0, 2.0, 3.0)];
        queue.begin_batch(&mut began);
        let first = queue.pop_event().expect("began");
        let second = queue.pop_event().expect("began");
        queue.end(sample(1, 1.5, 1.5, 3.1), IosTouchPhase::Cancelled);
        queue.cancel_all();
        let cancelled_native = queue.pop_event().expect("UIKit cancellation");
        let cancelled_teardown = queue.pop_event().expect("teardown cancellation");
        assert_eq!(cancelled_native.id, first.id);
        assert_eq!(cancelled_native.phase, IosTouchPhase::Cancelled);
        assert_eq!(cancelled_teardown.id, second.id);
        assert_eq!(cancelled_teardown.phase, IosTouchPhase::Cancelled);
        assert_eq!(cancelled_teardown.x_points, second.x_points);
        assert_eq!(queue.active_id(1), None);
        assert_eq!(queue.active_id(2), None);
        queue.begin(sample(3, 3.0, 3.0, 3.2));
        let restarted = queue
            .pop_event()
            .expect("input resumes after explicit cancellation");
        assert_eq!(restarted.phase, IosTouchPhase::Began);
        assert_eq!(restarted.id, 3);
    }

    #[test]
    fn invalid_coordinate_cancels_active_input_and_disables_delivery() {
        let mut queue = TouchQueue::new();
        queue.begin(sample(1, 1.0, 2.0, 4.0));
        let began = queue.pop_event().expect("began");
        queue.move_sample(sample(1, f64::NAN, 2.0, 4.1));
        assert_eq!(queue.fault(), Some(IosTouchInputFault::InvalidSample));
        let cancelled = queue
            .pop_event()
            .expect("invalid sample cancels active contact");
        assert_eq!(cancelled.id, began.id);
        assert_eq!(cancelled.phase, IosTouchPhase::Cancelled);
        queue.begin(sample(2, 0.0, 0.0, 4.2));
        assert_eq!(queue.pop_event(), None, "faulted input does not restart");
    }

    #[test]
    fn queue_capacity_overflow_cancels_active_input_and_is_sticky() {
        let mut queue = TouchQueue::new();
        queue.begin(sample(7, 1.0, 1.0, 5.0));
        for index in 0..MAX_QUEUED_TOUCH_EVENTS {
            queue.move_sample(sample(7, index as f64, 1.0, 5.1 + index as f64));
        }
        assert_eq!(
            queue.fault(),
            Some(IosTouchInputFault::EventCapacityExceeded)
        );
        let cancelled = queue.pop_event().expect("overflow cancellation");
        assert_eq!(cancelled.phase, IosTouchPhase::Cancelled);
        assert_eq!(cancelled.id, 1);
        assert_eq!(queue.pop_event(), None);
        assert_eq!(
            queue.fault(),
            Some(IosTouchInputFault::EventCapacityExceeded)
        );
    }

    #[test]
    fn contact_capacity_overflow_cancels_every_delivered_contact() {
        let mut queue = TouchQueue::new();
        for key in 0..MAX_ACTIVE_TOUCHES {
            queue.begin(sample(key + 1, key as f64, 1.0, 5.5));
        }
        while queue.pop_event().is_some() {}
        queue.begin(sample(MAX_ACTIVE_TOUCHES + 1, 0.0, 2.0, 5.6));
        assert_eq!(
            queue.fault(),
            Some(IosTouchInputFault::ContactCapacityExceeded)
        );
        let mut cancelled = 0;
        while let Some(event) = queue.pop_event() {
            assert_eq!(event.phase, IosTouchPhase::Cancelled);
            cancelled += 1;
        }
        assert_eq!(cancelled, MAX_ACTIVE_TOUCHES);
    }

    #[test]
    fn disabled_adapter_ignores_stale_callbacks_after_drop() {
        let mut queue = TouchQueue::new();
        queue.begin(sample(9, 4.0, 6.0, 6.0));
        let began = queue.pop_event().expect("began");
        queue.disable();
        let cancelled = queue.pop_event().expect("drop cancellation");
        assert_eq!(cancelled.id, began.id);
        assert_eq!(cancelled.phase, IosTouchPhase::Cancelled);
        queue.move_sample(sample(9, 8.0, 9.0, 6.1));
        assert_eq!(queue.pop_event(), None);
    }

    #[test]
    fn all_public_touch_kinds_and_pointer_metadata_are_preserved() {
        let kinds = [
            IosTouchKind::Direct,
            IosTouchKind::Indirect,
            IosTouchKind::Pencil,
            IosTouchKind::IndirectPointer,
            IosTouchKind::Other(71),
        ];
        let mut queue = TouchQueue::new();
        for (index, kind) in kinds.into_iter().enumerate() {
            let mut input = sample(index + 1, index as f64, 0.0, 7.0 + index as f64);
            input.kind = kind;
            input.tap_count = index as u32 + 1;
            input.button_mask = 1;
            queue.begin(input);
            let event = queue.pop_event().expect("contact metadata");
            assert_eq!(event.kind, kind);
            assert_eq!(event.tap_count, index as u32 + 1);
            assert_eq!(event.button_mask, 1);
            queue.end(input, IosTouchPhase::Ended);
            let _ended = queue.pop_event().expect("contact terminal");
        }
    }
}
