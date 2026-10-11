use core::future::{Future, Ready, ready};
use core::pin::pin;
use core::task::{Context, Poll, Waker};
use framework_core::{Availability, Error, ErrorKind};
use framework_location::{
    AccuracyTarget, Coordinate, Location, LocationAuthorization, LocationBackend, LocationError,
    LocationFix, LocationRequest, LocationTimestamp, LocationUpdate, LocationUpdateKind,
    LocationUpdateSession, LocationUpdatesBackend,
};
use std::collections::VecDeque;

struct Backend {
    authorization: LocationAuthorization,
    authorization_requests: u32,
    updates_started: u32,
    last_request: Option<LocationRequest>,
    events: VecDeque<LocationUpdate>,
}

impl Backend {
    fn new(authorization: LocationAuthorization, events: Vec<LocationUpdate>) -> Self {
        Self {
            authorization,
            authorization_requests: 0,
            updates_started: 0,
            last_request: None,
            events: events.into(),
        }
    }
}

impl LocationBackend for Backend {
    fn availability(&self) -> Availability {
        Availability::Available
    }

    type AuthorizationFuture<'a>
        = Ready<Result<LocationAuthorization, LocationError>>
    where
        Self: 'a;

    fn authorization<'a>(&'a mut self) -> Self::AuthorizationFuture<'a> {
        ready(Ok(self.authorization))
    }

    type RequestAuthorizationFuture<'a>
        = Ready<Result<LocationAuthorization, LocationError>>
    where
        Self: 'a;

    fn request_authorization<'a>(&'a mut self) -> Self::RequestAuthorizationFuture<'a> {
        self.authorization_requests += 1;
        ready(Ok(self.authorization))
    }

    type CurrentFuture<'a>
        = Ready<Result<LocationFix, LocationError>>
    where
        Self: 'a;

    fn current<'a>(&'a mut self, _request: LocationRequest) -> Self::CurrentFuture<'a> {
        ready(Err(LocationError::Backend(Error::new(
            ErrorKind::Unavailable,
        ))))
    }
}

#[derive(Debug)]
struct Session {
    events: VecDeque<LocationUpdate>,
}

impl LocationUpdateSession for Session {
    type NextUpdateFuture<'a>
        = Ready<LocationUpdate>
    where
        Self: 'a;

    fn next_update<'a>(&'a mut self) -> Self::NextUpdateFuture<'a> {
        ready(self.events.pop_front().expect("test event configured"))
    }
}

impl LocationUpdatesBackend for Backend {
    type UpdateSession<'a>
        = Session
    where
        Self: 'a;

    fn start_updates<'a>(
        &'a mut self,
        request: LocationRequest,
    ) -> Result<Self::UpdateSession<'a>, LocationError> {
        if !self.authorization.allows_foreground() {
            return Err(LocationError::Backend(Error::new(
                ErrorKind::PermissionDenied,
            )));
        }
        self.updates_started += 1;
        self.last_request = Some(request);
        Ok(Session {
            events: core::mem::take(&mut self.events),
        })
    }
}

fn run_ready<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("ready test backend returned pending"),
    }
}

fn request() -> LocationRequest {
    LocationRequest::new(AccuracyTarget::new(25.0).unwrap())
}

#[test]
fn updates_require_existing_authorization_without_requesting_it() {
    let mut location = Location::new(Backend::new(
        LocationAuthorization::NotDetermined,
        Vec::new(),
    ));
    assert_eq!(
        location.updates(request()).unwrap_err(),
        LocationError::Backend(Error::new(ErrorKind::PermissionDenied))
    );
    assert_eq!(location.backend().authorization_requests, 0);
    assert_eq!(location.backend().updates_started, 0);
}

#[test]
fn update_errors_are_events_and_do_not_prevent_later_fixes() {
    let fix = LocationFix::new(
        Coordinate::new(43.6532, -79.3832).unwrap(),
        12.5,
        LocationTimestamp::from_unix_millis(1_700_000_000_000),
    )
    .unwrap();
    let error = LocationError::Backend(Error::new(ErrorKind::Unavailable));
    let expected_error = LocationUpdate::new(LocationUpdateKind::Error(error), 0);
    let expected_fix = LocationUpdate::new(LocationUpdateKind::Fix(fix), 3);
    let mut location = Location::new(Backend::new(
        LocationAuthorization::Foreground,
        vec![expected_error, expected_fix],
    ));
    let mut updates = location.updates(request()).unwrap();
    assert_eq!(run_ready(updates.next_update()), expected_error);
    assert_eq!(run_ready(updates.next_update()), expected_fix);
    drop(updates);
    assert_eq!(location.backend().updates_started, 1);
    assert_eq!(location.backend().last_request, Some(request()));
    assert_eq!(location.backend().authorization_requests, 0);
}
