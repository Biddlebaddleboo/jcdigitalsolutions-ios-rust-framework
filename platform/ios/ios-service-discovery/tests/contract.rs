use core::future::{Future, Ready, ready};
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;

use framework_service_discovery::{
    BonjourAdvertisement, BonjourAdvertisementError, BonjourBrowseError, BonjourBrowser,
    BonjourBrowserBackend, BonjourBrowserEvent, BonjourBrowserState, BonjourQuery, BonjourService,
};

struct FakeBrowserBackend {
    events: VecDeque<BonjourBrowserEvent>,
    cancel_count: Rc<Cell<u32>>,
}

impl BonjourBrowserBackend for FakeBrowserBackend {
    type StartFuture<'a> = Ready<Result<(), BonjourBrowseError>>;
    type NextEventFuture<'a> = Ready<Result<BonjourBrowserEvent, BonjourBrowseError>>;
    type CloseFuture<'a> = Ready<Result<(), BonjourBrowseError>>;

    fn start<'a>(&'a mut self, _query: BonjourQuery) -> Self::StartFuture<'a> {
        ready(Ok(()))
    }

    fn next_event<'a>(&'a mut self) -> Self::NextEventFuture<'a> {
        ready(self.events.pop_front().ok_or(BonjourBrowseError::Cancelled))
    }

    fn close<'a>(&'a mut self) -> Self::CloseFuture<'a> {
        self.cancel();
        ready(Ok(()))
    }

    fn cancel(&mut self) {
        self.cancel_count.set(self.cancel_count.get() + 1);
    }
}

fn poll_ready<F: Future + Unpin>(mut future: F) -> F::Output {
    let mut context = Context::from_waker(Waker::noop());
    match Pin::new(&mut future).poll(&mut context) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("fake browser future unexpectedly pending"),
    }
}

#[test]
fn query_rejects_invalid_and_over_bound_service_types_and_domains() {
    assert_eq!(
        BonjourQuery::new("", None).unwrap_err(),
        BonjourBrowseError::InvalidServiceType
    );
    assert_eq!(
        BonjourQuery::new("_demo._sctp", None).unwrap_err(),
        BonjourBrowseError::InvalidServiceType
    );
    assert_eq!(
        BonjourQuery::new("_printer._sub._ipp._tcp", None).unwrap_err(),
        BonjourBrowseError::InvalidServiceType
    );
    assert_eq!(
        BonjourQuery::new("_demo._tcp\0", None).unwrap_err(),
        BonjourBrowseError::InvalidServiceType
    );
    assert_eq!(
        BonjourQuery::new("_münich._tcp", None).unwrap_err(),
        BonjourBrowseError::InvalidServiceType
    );
    assert_eq!(
        BonjourQuery::new("_demo._tcp", Some("local..")).unwrap_err(),
        BonjourBrowseError::InvalidDomain
    );
    let oversized = format!("_a.{}._tcp", "_b.".repeat(85));
    assert_eq!(
        BonjourQuery::new(&oversized, None).unwrap_err(),
        BonjourBrowseError::InvalidServiceType
    );
}

#[test]
fn selected_query_and_service_identity_preserve_exact_values() {
    let query = BonjourQuery::new("_speaker._tcp", Some("local.")).unwrap();
    assert_eq!(query.service_type(), "_speaker._tcp");
    assert_eq!(query.domain(), Some("local."));

    let service = BonjourService::new("Café speaker", "_speaker._tcp", "local.").unwrap();
    assert_eq!(service.name(), "Café speaker");
    assert_eq!(service.service_type(), "_speaker._tcp");
    assert_eq!(service.domain(), "local.");
}

#[test]
fn native_identity_rejects_invalid_utf8_and_over_bound_components() {
    assert_eq!(
        BonjourService::from_utf8_components(&[0xff], b"_speaker._tcp", b"local.").unwrap_err(),
        BonjourBrowseError::InvalidNativeService
    );
    assert_eq!(
        BonjourService::from_utf8_components(b"Speaker", b"_speaker._tcp", &vec![b'a'; 256])
            .unwrap_err(),
        BonjourBrowseError::InvalidNativeService
    );
}

#[test]
fn advertisement_preserves_values_and_uses_tcp_listener_port() {
    let advertisement =
        BonjourAdvertisement::new(Some("Café service"), "_demo._tcp", Some("local.")).unwrap();
    assert_eq!(advertisement.name(), Some("Café service"));
    assert_eq!(advertisement.service_type(), "_demo._tcp");
    assert_eq!(advertisement.domain(), Some("local."));

    let defaults = BonjourAdvertisement::new(None, "_demo._tcp", None).unwrap();
    assert_eq!(defaults.name(), None);
    assert_eq!(defaults.domain(), None);
}

#[test]
fn advertisement_rejects_udp_invalid_text_and_dns_sd_over_bounds() {
    assert_eq!(
        BonjourAdvertisement::new(None, "_demo._udp", None).unwrap_err(),
        BonjourAdvertisementError::InvalidServiceType
    );
    assert_eq!(
        BonjourAdvertisement::new(Some(""), "_demo._tcp", None).unwrap_err(),
        BonjourAdvertisementError::InvalidName
    );
    assert_eq!(
        BonjourAdvertisement::new(Some("bad\0name"), "_demo._tcp", None).unwrap_err(),
        BonjourAdvertisementError::InvalidName
    );
    assert_eq!(
        BonjourAdvertisement::new(Some(&"n".repeat(64)), "_demo._tcp", None).unwrap_err(),
        BonjourAdvertisementError::InvalidName
    );
    assert_eq!(
        BonjourAdvertisement::new(None, "_printer._sub._ipp._tcp", None).unwrap_err(),
        BonjourAdvertisementError::InvalidServiceType
    );
    assert_eq!(
        BonjourAdvertisement::new(None, "_demo._tcp", Some("local..")).unwrap_err(),
        BonjourAdvertisementError::InvalidDomain
    );
    let domain_253 = format!(
        "{}.{}.{}.{}",
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(61)
    );
    assert_eq!(domain_253.len(), 253);
    assert!(BonjourAdvertisement::new(None, "_demo._tcp", Some(&domain_253)).is_ok());
    assert!(BonjourAdvertisement::new(None, "_demo._tcp", Some(&format!("{domain_253}."))).is_ok());
    let domain_254 = format!(
        "{}.{}.{}.{}",
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(62)
    );
    assert_eq!(domain_254.len(), 254);
    assert_eq!(
        BonjourAdvertisement::new(None, "_demo._tcp", Some(&domain_254)).unwrap_err(),
        BonjourAdvertisementError::InvalidDomain
    );
}

#[test]
fn browser_facade_delivers_membership_and_coalesced_state_without_protocol_data() {
    let cancel_count = Rc::new(Cell::new(0));
    let backend = FakeBrowserBackend {
        events: VecDeque::from([
            BonjourBrowserEvent::ServiceAdded(
                BonjourService::new("Speaker", "_speaker._tcp", "local.").unwrap(),
            ),
            BonjourBrowserEvent::ServiceRemoved(
                BonjourService::new("Speaker", "_speaker._tcp", "local.").unwrap(),
            ),
            BonjourBrowserEvent::State(BonjourBrowserState::Waiting(None)),
        ]),
        cancel_count: Rc::clone(&cancel_count),
    };
    let mut browser = BonjourBrowser::new(backend);
    let query = BonjourQuery::new("_speaker._tcp", None).unwrap();
    assert_eq!(poll_ready(browser.start(query)), Ok(()));
    assert_eq!(
        poll_ready(browser.next_event()),
        Ok(BonjourBrowserEvent::ServiceAdded(
            BonjourService::new("Speaker", "_speaker._tcp", "local.").unwrap()
        ))
    );
    assert_eq!(
        poll_ready(browser.next_event()),
        Ok(BonjourBrowserEvent::ServiceRemoved(
            BonjourService::new("Speaker", "_speaker._tcp", "local.").unwrap()
        ))
    );
    assert_eq!(
        poll_ready(browser.next_event()),
        Ok(BonjourBrowserEvent::State(BonjourBrowserState::Waiting(
            None
        )))
    );
    drop(browser);
    assert_eq!(cancel_count.get(), 1);
}
