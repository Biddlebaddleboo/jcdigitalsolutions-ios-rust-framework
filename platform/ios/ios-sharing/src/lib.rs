#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(target_os = "ios"), allow(dead_code))]
#![doc = "Plain-text clipboard access and outgoing system sharing through public iOS UIKit APIs."]

extern crate alloc;

mod conversion;
mod operation;
mod share_conversion;
mod share_operation;

#[cfg(target_os = "ios")]
mod platform;
#[cfg(target_os = "ios")]
mod share_platform;

#[cfg(target_os = "ios")]
pub use platform::{
    IosClipboardBackend, IosClipboardClearFuture, IosClipboardReadFuture, IosClipboardWriteFuture,
};
#[cfg(target_os = "ios")]
pub use share_platform::{IosShareBackend, IosShareFuture};

#[cfg(test)]
mod tests {
    use super::conversion::decode_utf8;
    use super::operation::{Clear, ClipboardAccess, Deferred, Read, Write};
    use alloc::{string::String, vec};
    use core::future::Future;
    use core::pin::pin;
    use core::task::{Context, Poll, Waker};
    use framework_core::ErrorKind;
    use framework_sharing::ClipboardError;

    #[derive(Default)]
    struct FakeClipboard {
        value: Option<String>,
        reads: usize,
        writes: usize,
        clears: usize,
    }

    impl ClipboardAccess for FakeClipboard {
        fn read(&mut self) -> Result<Option<String>, ClipboardError> {
            self.reads += 1;
            Ok(self.value.clone())
        }

        fn write(&mut self, text: &str) -> Result<(), ClipboardError> {
            self.writes += 1;
            self.value = Some(String::from(text));
            Ok(())
        }

        fn clear(&mut self) -> Result<(), ClipboardError> {
            self.clears += 1;
            self.value = None;
            Ok(())
        }
    }

    fn poll_once<F: Future>(future: core::pin::Pin<&mut F>) -> Poll<F::Output> {
        future.poll(&mut Context::from_waker(Waker::noop()))
    }

    #[test]
    fn conversion_owns_valid_utf8_and_rejects_invalid_bytes() {
        assert_eq!(
            decode_utf8(vec![0x63, 0x61, 0x66, 0xc3, 0xa9]),
            Ok(String::from("café"))
        );
        let error = decode_utf8(vec![0xff]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn operations_start_on_first_poll_and_only_run_once() {
        let mut clipboard = FakeClipboard {
            value: Some(String::from("old")),
            ..FakeClipboard::default()
        };
        {
            let _future = Deferred::new(&mut clipboard, Write("new"));
        }
        {
            let _future = Deferred::new(&mut clipboard, Read);
        }
        {
            let _future = Deferred::new(&mut clipboard, Clear);
        }
        assert_eq!(clipboard.reads, 0);
        assert_eq!(clipboard.writes, 0);
        assert_eq!(clipboard.clears, 0);
        assert_eq!(clipboard.value.as_deref(), Some("old"));

        {
            let mut future = pin!(Deferred::new(&mut clipboard, Write("new")));
            assert_eq!(poll_once(future.as_mut()), Poll::Ready(Ok(())));
            assert_eq!(poll_once(future.as_mut()), Poll::Pending);
        }
        assert_eq!(clipboard.writes, 1);
        assert_eq!(clipboard.value.as_deref(), Some("new"));
    }

    #[test]
    fn read_and_clear_use_the_same_deferred_operation_seam() {
        let mut clipboard = FakeClipboard {
            value: Some(String::from("plain")),
            ..FakeClipboard::default()
        };
        {
            let mut read = pin!(Deferred::new(&mut clipboard, Read));
            assert_eq!(
                poll_once(read.as_mut()),
                Poll::Ready(Ok(Some(String::from("plain"))))
            );
        }
        assert_eq!(clipboard.reads, 1);

        {
            let mut clear = pin!(Deferred::new(&mut clipboard, Clear));
            assert_eq!(poll_once(clear.as_mut()), Poll::Ready(Ok(())));
        }
        assert_eq!(clipboard.clears, 1);
        assert_eq!(clipboard.value, None);
    }
}
