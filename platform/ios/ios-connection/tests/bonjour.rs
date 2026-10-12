use core::future::{Ready, ready};
use core::num::{NonZeroU16, NonZeroU32};
use std::cell::RefCell;
use std::rc::Rc;

use framework_connection::{
    AcceptedConnectionBackend, ConnectionBackend, ConnectionError, Endpoint, ReadChunk,
    TcpListener, TcpListenerAdvertisementBackend, TcpListenerBackend,
};

#[derive(Default)]
struct ListenerState {
    advertisement: Option<&'static str>,
    cancelled: bool,
}

struct FakeListener(Rc<RefCell<ListenerState>>);

struct FakeStream;

impl ConnectionBackend for FakeStream {
    type ConnectFuture<'a>
        = Ready<Result<(), ConnectionError>>
    where
        Self: 'a;
    type SendFuture<'a>
        = Ready<Result<(), ConnectionError>>
    where
        Self: 'a;
    type ReceiveFuture<'a>
        = Ready<Result<ReadChunk, ConnectionError>>
    where
        Self: 'a;
    type CloseFuture<'a>
        = Ready<Result<(), ConnectionError>>
    where
        Self: 'a;

    fn connect<'a>(&'a mut self, _endpoint: Endpoint) -> Self::ConnectFuture<'a> {
        ready(Ok(()))
    }

    fn send<'a>(&'a mut self, _bytes: &'a [u8]) -> Self::SendFuture<'a> {
        ready(Ok(()))
    }

    fn receive<'a>(&'a mut self, _max_bytes: NonZeroU32) -> Self::ReceiveFuture<'a> {
        ready(Err(ConnectionError::BackendFailure))
    }

    fn close<'a>(&'a mut self) -> Self::CloseFuture<'a> {
        ready(Ok(()))
    }

    fn cancel(&mut self) {}
}

impl AcceptedConnectionBackend for FakeStream {}

impl TcpListenerBackend for FakeListener {
    type AcceptedBackend = FakeStream;
    type BindFuture<'a>
        = Ready<Result<NonZeroU16, ConnectionError>>
    where
        Self: 'a;
    type AcceptFuture<'a>
        = Ready<Result<FakeStream, ConnectionError>>
    where
        Self: 'a;
    type CloseFuture<'a>
        = Ready<Result<(), ConnectionError>>
    where
        Self: 'a;

    fn bind<'a>(&'a mut self, _port: u16) -> Self::BindFuture<'a> {
        ready(Ok(NonZeroU16::new(1).unwrap()))
    }

    fn accept<'a>(&'a mut self) -> Self::AcceptFuture<'a> {
        ready(Err(ConnectionError::BackendFailure))
    }

    fn close<'a>(&'a mut self) -> Self::CloseFuture<'a> {
        ready(Ok(()))
    }

    fn cancel(&mut self) {
        let mut state = self.0.borrow_mut();
        state.cancelled = true;
        state.advertisement = None;
    }
}

impl TcpListenerAdvertisementBackend for FakeListener {
    type Advertisement = &'static str;

    fn register_advertisement(
        &mut self,
        advertisement: Self::Advertisement,
    ) -> Result<(), ConnectionError> {
        let mut state = self.0.borrow_mut();
        if state.cancelled || state.advertisement.is_some() {
            return Err(ConnectionError::InvalidState);
        }
        state.advertisement = Some(advertisement);
        Ok(())
    }

    fn update_advertisement(
        &mut self,
        advertisement: Self::Advertisement,
    ) -> Result<(), ConnectionError> {
        let mut state = self.0.borrow_mut();
        if state.cancelled || state.advertisement.is_none() {
            return Err(ConnectionError::InvalidState);
        }
        state.advertisement = Some(advertisement);
        Ok(())
    }

    fn stop_advertising(&mut self) -> Result<(), ConnectionError> {
        self.0.borrow_mut().advertisement = None;
        Ok(())
    }
}

#[test]
fn listener_advertisement_lifecycle() {
    let state = Rc::new(RefCell::new(ListenerState::default()));
    let mut listener = TcpListener::new(FakeListener(Rc::clone(&state)));

    assert_eq!(
        listener.update_advertisement("_demo._tcp"),
        Err(ConnectionError::InvalidState)
    );
    listener.register_advertisement("_demo._tcp").unwrap();
    assert_eq!(state.borrow().advertisement, Some("_demo._tcp"));
    assert_eq!(
        listener.register_advertisement("_other._tcp"),
        Err(ConnectionError::InvalidState)
    );
    listener.update_advertisement("_other._tcp").unwrap();
    assert_eq!(state.borrow().advertisement, Some("_other._tcp"));
    listener.stop_advertising().unwrap();
    listener.stop_advertising().unwrap();
    assert_eq!(state.borrow().advertisement, None);
    assert!(!state.borrow().cancelled);

    listener.register_advertisement("_demo._tcp").unwrap();
    drop(listener);
    assert!(state.borrow().cancelled);
    assert_eq!(state.borrow().advertisement, None);
}
