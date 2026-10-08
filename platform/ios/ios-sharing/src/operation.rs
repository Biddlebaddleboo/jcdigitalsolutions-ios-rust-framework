use alloc::rc::Rc;
use alloc::string::String;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use framework_sharing::ClipboardError;

pub(crate) trait ClipboardAccess {
    fn read(&mut self) -> Result<Option<String>, ClipboardError>;
    fn write(&mut self, text: &str) -> Result<(), ClipboardError>;
    fn clear(&mut self) -> Result<(), ClipboardError>;
}

pub(crate) trait Start<A: ClipboardAccess> {
    type Output;

    fn start(self, access: &mut A) -> Self::Output;
}

pub(crate) struct Read;

impl<A: ClipboardAccess> Start<A> for Read {
    type Output = Result<Option<String>, ClipboardError>;

    fn start(self, access: &mut A) -> Self::Output {
        access.read()
    }
}

pub(crate) struct Write<'a>(pub(crate) &'a str);

impl<A: ClipboardAccess> Start<A> for Write<'_> {
    type Output = Result<(), ClipboardError>;

    fn start(self, access: &mut A) -> Self::Output {
        access.write(self.0)
    }
}

pub(crate) struct Clear;

impl<A: ClipboardAccess> Start<A> for Clear {
    type Output = Result<(), ClipboardError>;

    fn start(self, access: &mut A) -> Self::Output {
        access.clear()
    }
}

pub(crate) struct Deferred<'a, A: ClipboardAccess, O: Start<A>> {
    access: Option<&'a mut A>,
    operation: Option<O>,
    _not_send: PhantomData<Rc<()>>,
}

impl<'a, A: ClipboardAccess, O: Start<A>> Deferred<'a, A, O> {
    pub(crate) fn new(access: &'a mut A, operation: O) -> Self {
        Self {
            access: Some(access),
            operation: Some(operation),
            _not_send: PhantomData,
        }
    }
}

impl<A: ClipboardAccess, O: Start<A> + Unpin> Future for Deferred<'_, A, O> {
    type Output = O::Output;

    fn poll(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let (Some(access), Some(operation)) = (this.access.take(), this.operation.take()) else {
            return Poll::Pending;
        };
        Poll::Ready(operation.start(access))
    }
}
