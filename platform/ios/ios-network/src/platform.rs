use crate::conversion::{NativeErrorClass, OwnedRequest, response_from_parts};
use crate::operation::CompletionCell;
use alloc::vec::Vec;
use block2::RcBlock;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use framework_core::{Availability, Error, ErrorKind};
use framework_network::{HttpBackend, HttpRequest, HttpResponse, NetworkError};
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::AnyObject;
use objc2_foundation::{
    NSData, NSError, NSHTTPURLResponse, NSMutableURLRequest, NSString, NSURL, NSURLErrorCancelled,
    NSURLErrorCannotConnectToHost, NSURLErrorCannotFindHost, NSURLErrorDomain,
    NSURLErrorNetworkConnectionLost, NSURLErrorNotConnectedToInternet, NSURLErrorTimedOut,
    NSURLResponse, NSURLSession, NSURLSessionConfiguration, NSURLSessionDataTask,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

/// An executor-neutral foreground HTTP backend backed by Foundation URLSession.
pub struct IosHttpBackend {
    session: Retained<NSURLSession>,
    invalidate_on_drop: bool,
}

impl IosHttpBackend {
    /// Creates a private session with Apple's default foreground configuration.
    ///
    /// The backend calls `finishTasksAndInvalidate` when it is dropped.
    pub fn new() -> Self {
        let session = autoreleasepool(|_| {
            let configuration = NSURLSessionConfiguration::defaultSessionConfiguration();
            NSURLSession::sessionWithConfiguration(&configuration)
        });
        Self {
            session,
            invalidate_on_drop: true,
        }
    }

    /// Creates a backend around an explicitly configured caller-owned native session.
    ///
    /// The caller retains responsibility for invalidating this session.
    pub fn with_session(session: Retained<NSURLSession>) -> Self {
        Self {
            session,
            invalidate_on_drop: false,
        }
    }

    /// Borrows the backend's native session for iOS-specific options and delegate-free operations.
    pub fn native_session(&self) -> &NSURLSession {
        &self.session
    }
}

impl Drop for IosHttpBackend {
    fn drop(&mut self) {
        if self.invalidate_on_drop {
            autoreleasepool(|_| self.session.finishTasksAndInvalidate());
        }
    }
}

impl Default for IosHttpBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpBackend for IosHttpBackend {
    fn availability(&self) -> Availability {
        Availability::Available
    }

    type SendFuture<'a> = IosSendFuture<'a>;

    fn send<'a>(&'a mut self, request: HttpRequest<'a>) -> Self::SendFuture<'a> {
        let request = OwnedRequest::from_request(request);
        IosSendFuture {
            session: &self.session,
            request: Some(request),
            completion: Arc::new(CompletionCell::new()),
            task: None,
            started: false,
            finished: false,
        }
    }
}

/// A lazy one-shot request future; its URLSession data task starts on first poll.
pub struct IosSendFuture<'a> {
    session: &'a NSURLSession,
    request: Option<Result<OwnedRequest, NetworkError>>,
    completion: Arc<CompletionCell<HttpResponse, NetworkError>>,
    task: Option<Retained<NSURLSessionDataTask>>,
    started: bool,
    finished: bool,
}

impl IosSendFuture<'_> {
    fn start(&mut self) {
        if self.started {
            return;
        }
        self.started = true;
        let Some(request) = self.request.take() else {
            self.completion
                .complete(Err(NetworkError::Backend(Error::new(ErrorKind::Internal))));
            return;
        };
        let request = match request {
            Ok(request) => request,
            Err(error) => {
                self.completion.complete(Err(error));
                return;
            }
        };
        let request = match catch_unwind(AssertUnwindSafe(|| make_request(request))) {
            Ok(Ok(request)) => request,
            Ok(Err(error)) => {
                self.completion.complete(Err(error));
                return;
            }
            Err(_) => {
                self.completion
                    .complete(Err(NetworkError::Backend(Error::new(ErrorKind::Internal))));
                return;
            }
        };
        let completion = self.completion.clone();
        let handler = RcBlock::new(move |data, response, error| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                autoreleasepool(|_| unsafe { response_from_callback(data, response, error) })
            }))
            .unwrap_or_else(|_| Err(NetworkError::Backend(Error::new(ErrorKind::Internal))));
            completion.complete(result);
        });
        let task = autoreleasepool(|_| {
            // SAFETY: The copied block captures only an `Arc<CompletionCell<...>>`. Its contents
            // are protected by a mutex, and neither the block nor callback captures non-Send
            // native data. URLSession retains the completion handler for the asynchronous task.
            let task = unsafe {
                self.session
                    .dataTaskWithRequest_completionHandler(&request, &handler)
            };
            task.resume();
            task
        });
        self.task = Some(task);
    }
}

impl Future for IosSendFuture<'_> {
    type Output = Result<HttpResponse, NetworkError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if !this.started {
            this.start();
        }
        match this.completion.poll(context) {
            Poll::Ready(result) => {
                this.finished = true;
                this.task = None;
                Poll::Ready(result)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for IosSendFuture<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.completion.detach();
            if let Some(task) = self.task.take() {
                autoreleasepool(|_| task.cancel());
            }
        }
    }
}

fn make_request(request: OwnedRequest) -> Result<Retained<NSMutableURLRequest>, NetworkError> {
    autoreleasepool(|_| {
        let url_text = NSString::from_str(&request.url);
        let url = NSURL::URLWithString(&url_text).ok_or(NetworkError::InvalidUrl)?;
        if url.host().is_none() {
            return Err(NetworkError::InvalidUrl);
        }
        let native = NSMutableURLRequest::requestWithURL(&url);
        let method = NSString::from_str(&request.method);
        native.setHTTPMethod(&method);
        for header in request.headers {
            let name = NSString::from_str(&header.name);
            let value = NSString::from_str(&header.value);
            native.addValue_forHTTPHeaderField(&value, &name);
        }
        if let Some(body) = request.body {
            native.setHTTPBody(Some(&NSData::from_vec(body)));
        }
        Ok(native)
    })
}

unsafe fn response_from_callback(
    data: *mut NSData,
    response: *mut NSURLResponse,
    error: *mut NSError,
) -> Result<HttpResponse, NetworkError> {
    // SAFETY: URLSession's completion handler pointers remain valid for the callback duration;
    // nullable Objective-C results are represented as null pointers by objc2-foundation.
    if let Some(error) = unsafe { error.as_ref() } {
        return Err(map_nserror(error));
    }
    // SAFETY: The callback response pointer is borrowed and valid for this invocation.
    let Some(response) = (unsafe { response.cast::<AnyObject>().as_ref() }) else {
        return Err(NetworkError::Backend(Error::new(ErrorKind::Platform)));
    };
    let Some(response) = response.downcast_ref::<NSHTTPURLResponse>() else {
        return Err(NetworkError::Backend(Error::new(ErrorKind::Platform)));
    };
    let status = i64::try_from(response.statusCode()).unwrap_or(i64::MAX);
    let fields = response.allHeaderFields();
    let (keys, values) = fields.to_vecs();
    let mut headers = Vec::with_capacity(keys.len());
    for (key, value) in keys.into_iter().zip(values) {
        let Ok(key) = key.downcast::<NSString>() else {
            return Err(NetworkError::Backend(Error::new(ErrorKind::Platform)));
        };
        let Ok(value) = value.downcast::<NSString>() else {
            return Err(NetworkError::Backend(Error::new(ErrorKind::Platform)));
        };
        headers.push((key.to_string(), value.to_string()));
    }
    // SAFETY: URLSession lends the body NSData for the callback duration. `to_vec` copies bytes
    // before the callback returns; a null body with a valid response represents an empty body.
    let body = unsafe { data.as_ref() }.map_or_else(Vec::new, NSData::to_vec);
    response_from_parts(status, headers, body)
}

#[allow(non_upper_case_globals)]
fn map_nserror(error: &NSError) -> NetworkError {
    let domain = error.domain();
    let code = error.code();
    // SAFETY: `NSURLErrorDomain` is Foundation's process-lifetime public domain constant.
    let url_error_domain = unsafe { NSURLErrorDomain };
    let class = if &*domain == url_error_domain {
        match code {
            NSURLErrorCancelled => NativeErrorClass::Cancelled,
            NSURLErrorTimedOut => NativeErrorClass::Timeout,
            NSURLErrorCannotFindHost
            | NSURLErrorCannotConnectToHost
            | NSURLErrorNetworkConnectionLost
            | NSURLErrorNotConnectedToInternet => NativeErrorClass::Unavailable,
            _ => NativeErrorClass::Other,
        }
    } else {
        NativeErrorClass::Other
    };
    let code = i64::try_from(code).unwrap_or(i64::MAX);
    crate::conversion::native_error(class, code)
}
