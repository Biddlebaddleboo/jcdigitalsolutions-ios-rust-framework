#[cfg(test)]
use crate::geofence::GEOFENCE_EVENT_QUEUE_CAPACITY;
use crate::geofence::{
    GeofenceEventFuture, GeofenceEventQueue, IosGeofence, IosGeofenceError, IosGeofenceEvent,
    MonitorLease, OpenCompletionCell, geofence_issues_from_flags, geofence_name_is_valid,
    geofence_state_from_native,
};
use block2::RcBlock;
use core::future::Future;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll};
use dispatch2::{
    DispatchQueue, DispatchQueueGlobalPriority, DispatchRetained, GlobalQueueIdentifier,
};
use objc2::rc::{Retained, autoreleasepool};
use objc2::{AnyThread, MainThreadMarker, available};
use objc2_core_location::{
    CLCircularGeographicCondition, CLLocationCoordinate2D, CLMonitor, CLMonitorConfiguration,
    CLMonitoringEvent, CLMonitoringRecord,
};
use objc2_foundation::{NSArray, NSDate, NSString};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::Waker;

struct MonitorShared {
    queue: DispatchRetained<DispatchQueue>,
    monitor: Mutex<Option<Retained<CLMonitor>>>,
    events: Arc<GeofenceEventQueue>,
    _lease: MonitorLease,
}

impl MonitorShared {
    fn monitor_lock(&self) -> MutexGuard<'_, Option<Retained<CLMonitor>>> {
        self.monitor
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    fn close_on_queue(&self) {
        self.queue.exec_sync(|| {
            let monitor = self.monitor_lock().take();
            drop(monitor);
        });
    }

    fn with_monitor<T: Send>(
        &self,
        operation: impl FnOnce(&CLMonitor) -> T + Send,
    ) -> Result<T, IosGeofenceError> {
        let mut output = None;
        self.queue.exec_sync(|| {
            autoreleasepool(|_| {
                let monitor = self.monitor_lock();
                if let Some(monitor) = monitor.as_deref() {
                    output = Some(operation(monitor));
                }
            });
        });
        output.ok_or(IosGeofenceError::NativeFailure)
    }
}

/// A lazy open request for one persistent, caller-named Core Location monitor.
pub struct IosGeofenceMonitorFuture {
    name: String,
    marker: MainThreadMarker,
    shared: Option<Arc<MonitorShared>>,
    completion: Arc<OpenCompletionCell>,
    started: bool,
    delivered: bool,
}

/// A caller-owned persistent geofence monitor with queue-confined native access.
pub struct IosGeofenceMonitor {
    shared: Arc<MonitorShared>,
}

impl IosGeofenceMonitorFuture {
    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        let Some(shared) = self.shared.as_ref().cloned() else {
            let _ = self
                .completion
                .complete_with(Err(IosGeofenceError::NativeFailure), || {});
            return;
        };
        let completion_state = Arc::new(PendingOpen {
            shared: shared.clone(),
            completion: self.completion.clone(),
        });
        let event_queue = shared.events.clone();
        let event_handler = RcBlock::new(
            move |monitor: NonNull<CLMonitor>, event: NonNull<CLMonitoringEvent>| {
                let _monitor = monitor;
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    let outcome = catch_unwind(AssertUnwindSafe(|| {
                        autoreleasepool(|_| unsafe { copy_native_event(event.as_ref()) })
                    }));
                    let event = match outcome {
                        Ok(Some(event)) => event,
                        Ok(None) | Err(_) => IosGeofenceEvent::callback_failure(),
                    };
                    wake_off_monitor_queue(event_queue.push_deferred(event));
                }));
            },
        );
        let name = NSString::from_str(&self.name);
        let configuration = unsafe {
            CLMonitorConfiguration::configWithMonitorName_queue_eventHandler(
                &name,
                shared.queue.as_ref(),
                &event_handler,
            )
        };
        let callback_state = completion_state.clone();
        let completion = RcBlock::new(move |monitor: NonNull<CLMonitor>| {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                // SAFETY: Core Location invokes this completion with its live monitor instance on
                // the configured queue. Retain it before the callback returns.
                let Some(monitor) = (unsafe { Retained::retain(monitor.as_ptr()) }) else {
                    let (_, waker) = callback_state
                        .completion
                        .complete_with_deferred(Err(IosGeofenceError::NativeFailure), || {});
                    wake_off_monitor_queue(waker);
                    return;
                };
                let mut monitor = Some(monitor);
                let (accepted, waker) =
                    callback_state
                        .completion
                        .complete_with_deferred(Ok(()), || {
                            if let Some(monitor) = monitor.take() {
                                let previous =
                                    callback_state.shared.monitor_lock().replace(monitor);
                                drop(previous);
                            }
                        });
                wake_off_monitor_queue(waker);
                if !accepted {
                    drop(monitor);
                }
            }));
            if outcome.is_err() {
                let (_, waker) = callback_state
                    .completion
                    .complete_with_deferred(Err(IosGeofenceError::CallbackFailure), || {});
                wake_off_monitor_queue(waker);
            }
        });
        // SAFETY: iOS 17 availability was checked before constructing this future. The
        // configuration binds every returned-monitor operation and event callback to its serial
        // queue, and both blocks are retained by Core Location for their documented lifetimes.
        unsafe {
            CLMonitor::requestMonitorWithConfiguration_completion(&configuration, &completion)
        };
        let _ = self.marker;
    }
}

impl Future for IosGeofenceMonitorFuture {
    type Output = Result<IosGeofenceMonitor, IosGeofenceError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if this.delivered {
            return Poll::Ready(Err(IosGeofenceError::NativeFailure));
        }
        let start = catch_unwind(AssertUnwindSafe(|| this.start()));
        if start.is_err() {
            let _ = this
                .completion
                .complete_with(Err(IosGeofenceError::CallbackFailure), || {});
        }
        match this.completion.poll(context) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => {
                this.delivered = true;
                match this.shared.take() {
                    Some(shared) => Poll::Ready(Ok(IosGeofenceMonitor { shared })),
                    None => Poll::Ready(Err(IosGeofenceError::NativeFailure)),
                }
            }
            Poll::Ready(Err(error)) => {
                this.delivered = true;
                if let Some(shared) = this.shared.take() {
                    shared.events.detach();
                    shared.close_on_queue();
                }
                Poll::Ready(Err(error))
            }
        }
    }
}

impl Drop for IosGeofenceMonitorFuture {
    fn drop(&mut self) {
        if !self.delivered {
            self.completion.detach();
            if let Some(shared) = self.shared.as_ref() {
                shared.events.detach();
                shared.close_on_queue();
            }
        }
    }
}

impl IosGeofenceMonitor {
    /// Adds a circular condition or replaces the condition with the same identifier.
    pub fn add_or_replace(&mut self, geofence: &IosGeofence) -> Result<(), IosGeofenceError> {
        let geofence = geofence.clone();
        self.shared.with_monitor(move |monitor| unsafe {
            let identifier = NSString::from_str(geofence.identifier());
            let condition = CLCircularGeographicCondition::initWithCenter_radius(
                CLCircularGeographicCondition::alloc(),
                CLLocationCoordinate2D {
                    latitude: geofence.center().latitude_degrees(),
                    longitude: geofence.center().longitude_degrees(),
                },
                geofence.radius_meters(),
            );
            monitor.addConditionForMonitoring_identifier(&condition, &identifier);
        })
    }

    /// Removes a persisted condition by its stable identifier.
    pub fn remove(&mut self, identifier: &str) -> Result<(), IosGeofenceError> {
        if identifier.is_empty() {
            return Err(IosGeofenceError::InvalidIdentifier);
        }
        let identifier = identifier.to_owned();
        self.shared.with_monitor(move |monitor| unsafe {
            monitor.removeConditionFromMonitoringWithIdentifier(&NSString::from_str(&identifier));
        })
    }

    /// Lists the circular conditions currently persisted by this monitor.
    pub fn list(&mut self) -> Result<Vec<IosGeofence>, IosGeofenceError> {
        self.shared.with_monitor(|monitor| unsafe {
            let identifiers: Retained<NSArray<NSString>> = monitor.monitoredIdentifiers();
            let mut regions = Vec::with_capacity(identifiers.len());
            for index in 0..identifiers.len() {
                let identifier = identifiers.objectAtIndex(index);
                let Some(record): Option<Retained<CLMonitoringRecord>> =
                    monitor.monitoringRecordForIdentifier(&identifier)
                else {
                    continue;
                };
                let condition = record.condition();
                let Ok(condition) = condition.downcast::<CLCircularGeographicCondition>() else {
                    return Err(IosGeofenceError::NativeFailure);
                };
                let center = condition.center();
                regions.push(IosGeofence::new(
                    identifier.to_string(),
                    center.latitude,
                    center.longitude,
                    condition.radius(),
                )?);
            }
            Ok(regions)
        })?
    }

    /// Returns a future for the next copied monitor event, or `None` after monitor drop.
    pub fn next_event(&mut self) -> GeofenceEventFuture<'_> {
        GeofenceEventFuture::new(&self.shared.events)
    }
}

impl Drop for IosGeofenceMonitor {
    fn drop(&mut self) {
        self.shared.events.detach();
        self.shared.close_on_queue();
    }
}

struct PendingOpen {
    shared: Arc<MonitorShared>,
    completion: Arc<OpenCompletionCell>,
}

pub(super) fn prepare_monitor_open(
    name: &str,
    marker: MainThreadMarker,
) -> Result<IosGeofenceMonitorFuture, IosGeofenceError> {
    if !geofence_name_is_valid(name) {
        return Err(IosGeofenceError::InvalidMonitorName);
    }
    let lease = MonitorLease::acquire()?;
    let queue = DispatchQueue::new("com.jcdigitalsolutions.ios-location.geofence", None);
    let shared = Arc::new(MonitorShared {
        queue,
        monitor: Mutex::new(None),
        events: Arc::new(GeofenceEventQueue::new()),
        _lease: lease,
    });
    Ok(IosGeofenceMonitorFuture {
        name: name.to_owned(),
        marker,
        shared: Some(shared),
        completion: Arc::new(OpenCompletionCell::new()),
        started: false,
        delivered: false,
    })
}

unsafe fn copy_native_event(event: &CLMonitoringEvent) -> Option<IosGeofenceEvent> {
    // SAFETY: Core Location exposes event fields as readonly properties valid for the callback.
    let identifier = unsafe { event.identifier() }.to_string();
    if identifier.is_empty() {
        return None;
    }
    // SAFETY: State and date are readonly event values valid during this callback.
    let state = geofence_state_from_native(unsafe { event.state() }.0);
    let date: Retained<NSDate> = unsafe { event.date() };
    let timestamp = unix_millis(date.timeIntervalSince1970());
    let mut issues = Vec::new();
    if available!(ios = 18.0, ..) {
        // SAFETY: Every issue property below was introduced in iOS 18.0 and is guarded here.
        let flags = unsafe {
            [
                event.authorizationDenied(),
                event.authorizationRestricted(),
                event.authorizationDeniedGlobally(),
                event.insufficientlyInUse(),
                event.accuracyLimited(),
                event.conditionUnsupported(),
                event.conditionLimitExceeded(),
                event.persistenceUnavailable(),
                event.serviceSessionRequired(),
                event.authorizationRequestInProgress(),
            ]
        };
        issues = geofence_issues_from_flags(flags);
    }
    Some(IosGeofenceEvent::new(
        Some(identifier),
        state,
        timestamp,
        issues,
    ))
}

fn unix_millis(seconds: f64) -> Option<u64> {
    let milliseconds = seconds * 1_000.0;
    if !milliseconds.is_finite() || milliseconds < 0.0 || milliseconds >= u64::MAX as f64 {
        None
    } else {
        Some(milliseconds as u64)
    }
}

fn wake_off_monitor_queue(waker: Option<Waker>) {
    let Some(waker) = waker else {
        return;
    };
    let queue = DispatchQueue::global_queue(GlobalQueueIdentifier::Priority(
        DispatchQueueGlobalPriority::Default,
    ));
    queue.exec_async(move || {
        let _ = catch_unwind(AssertUnwindSafe(|| waker.wake()));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_time_mapping_rejects_nonfinite_and_out_of_range_values() {
        assert_eq!(unix_millis(1.5), Some(1_500));
        assert_eq!(unix_millis(f64::NAN), None);
        assert_eq!(unix_millis(f64::INFINITY), None);
        assert_eq!(unix_millis(-1.0), None);
    }

    #[test]
    fn queue_capacity_contract_remains_sixteen() {
        assert_eq!(GEOFENCE_EVENT_QUEUE_CAPACITY, 16);
    }
}
