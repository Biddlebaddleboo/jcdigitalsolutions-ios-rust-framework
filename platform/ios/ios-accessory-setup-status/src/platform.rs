use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

use crate::AccessorySetupStatusError;
use block2::RcBlock;
use dispatch2::DispatchQueue;
use objc2::rc::Retained;
use objc2_accessory_setup_kit::{ASAccessoryEvent, ASAccessoryEventType, ASAccessorySession};

type Completion = Box<dyn FnOnce(Result<usize, AccessorySetupStatusError>) + Send + 'static>;

pub(super) fn request_previously_selected_accessory_count<F>(completion: F)
where
    F: FnOnce(Result<usize, AccessorySetupStatusError>) + Send + 'static,
{
    let completion: Arc<Mutex<Option<Completion>>> =
        Arc::new(Mutex::new(Some(Box::new(completion))));
    // SAFETY: the public API requires an iOS 18.0+ app deployment target
    let session = unsafe { ASAccessorySession::new() };
    let callback_session: Retained<ASAccessorySession> = session.clone();
    let callback_completion = Arc::clone(&completion);
    let event_handler = RcBlock::new(move |event: std::ptr::NonNull<ASAccessoryEvent>| {
        // SAFETY: AccessorySetupKit invokes the block with a live event object on the requested
        // main queue. Its public event type and the session's array are read on that same queue
        let event_type = unsafe { event.as_ref().eventType() };
        match event_type {
            ASAccessoryEventType::Activated => {
                // SAFETY: Apple documents the previously selected accessory array after activation;
                // this read occurs on the serialized queue used to activate the session
                let count = unsafe { callback_session.accessories() }.len();
                let callback = take_completion(&callback_completion);
                // SAFETY: Invalidation stops session operations and breaks native callback cycles
                unsafe { callback_session.invalidate() };
                invoke_completion(callback, Ok(count));
            }
            ASAccessoryEventType::Invalidated => invoke_completion(
                take_completion(&callback_completion),
                Err(AccessorySetupStatusError::SessionInvalidated),
            ),
            _ => {}
        }
    });

    // SAFETY: AccessorySetupKit invokes events on the supplied main queue. The block and captured
    // session remain retained by the native session until activation or invalidation completes
    unsafe { session.activateWithQueue_eventHandler(DispatchQueue::main(), &event_handler) };
}

fn take_completion(completion: &Arc<Mutex<Option<Completion>>>) -> Option<Completion> {
    completion
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}

fn invoke_completion(
    callback: Option<Completion>,
    result: Result<usize, AccessorySetupStatusError>,
) {
    if let Some(callback) = callback {
        let _ = catch_unwind(AssertUnwindSafe(|| callback(result)));
    }
}
