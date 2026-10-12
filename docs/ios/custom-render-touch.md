# Native custom-render touch input

ios-ui::IosCustomRenderTouchInput attaches one transparent UIKit UIView over an existing
IosCustomRenderHost. It uses touchesBegan:withEvent:, touchesMoved:withEvent:,
touchesEnded:withEvent:, and touchesCancelled:withEvent: from UIKit's documented responder
delivery. It does not add a gesture recognizer or infer taps, drags, pans, or other gesture policy.

Typical use:

    let mut touch_input = IosCustomRenderTouchInput::attach(&host, marker);
    while let Some(event) = touch_input.next_event() {
        // Apply raw contact samples to the caller's own input model.
    }

The adapter is UIKit-main-thread-bound. The event queue uses fixed capacity for 1,024 samples and
the active-contact table supports 64 contacts. Capacity for cancellation of every active contact is
reserved in the queue. If a limit is exceeded, it clears queued history, emits Cancelled for every
active sequence admitted from UIKit, sets a sticky fault(), and ignores later input. A cancellation
can therefore follow a Began event still pending in the queue; the sticky
fault signals that event history is discontinuous. Recreate the adapter to resume. The contact
that crossed a limit receives no event if its Began event was not admitted. Invalid, non-finite
coordinates or timestamps and invalid scales produce the same safe cancellation and sticky-fault
behavior.

Each admitted Began receives a nonzero u64 ID that remains stable through its Moved and terminal
event. IDs are local to one adapter instance. UIKit touch object addresses are used only as
temporary internal lookup keys; they are not exposed or retained. Event samples are immutable
copies. Events from one began set are ordered by host-local Y, then X, then the UIKit object
identity as a tie-break. Changed contacts in move and terminal callbacks are ordered by stable ID.
UIKit's callback order is otherwise retained.

timestamp_seconds is UIKit's monotonic UITouch.timestamp. Point coordinates are local to the
transparent overlay, whose bounds track the render host. Pixel coordinates are the point values
multiplied by the host's contentScaleFactor at sample time; the event carries that scale. A
native touchesCancelled callback is delivered as Cancelled. Indirect pointer button masks are
reported on iOS 13.4 and later; earlier systems report zero. If the overlay leaves its window,
all active contacts are cancelled using their last known sample positions and timestamps. Call
cancel_active() when the app or host loses input focus without a UIKit cancellation callback. Call
detach() before dropping the adapter when the caller must drain terminal events; Drop also disables
callbacks and removes the overlay, but the queue is no longer accessible after drop.

When UIKit returns coalesced samples for a moved touch, every supplied sample is emitted in the
provided array order with its own native timestamp and coalesced = true. If UIKit supplies no
coalesced array, one ordinary moved event is emitted. No predicted touches are emitted. Event
metadata includes UIKit touch kind, tap count, and UIEvent.buttonMask; a missing event has a zero
button mask. Keyboard modifier state and stylus pressure are not inferred.

The overlay participates in UIKit hit testing and handles touches over the render host. Native
interactive subviews added above it may receive those contacts instead. The consumer owns layering,
native gesture-recognizer policy, hit testing beyond the host, event draining cadence, and all
application meaning. Static compilation and link probes do not prove simulator or device touch
delivery; deterministic host tests cover the state machine and queue failure behavior.
