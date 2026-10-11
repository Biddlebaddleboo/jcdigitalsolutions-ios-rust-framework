#[cfg(target_os = "ios")]
fn main() {
    use ios_call_provider::{
        ActionDecision, CallEndReason, CallHandleKind, CallId, CallProvider, CallProviderConfig,
        IncomingCall,
    };
    use objc2::MainThreadMarker;

    let Some(main_thread) = MainThreadMarker::new() else {
        return;
    };
    let Ok(mut provider) =
        CallProvider::new(main_thread, CallProviderConfig::new("Link Probe"), |_| {
            ActionDecision::Fail
        })
    else {
        return;
    };
    let call_id = CallId::from_bytes([1; 16]);
    let call = IncomingCall::new(call_id, CallHandleKind::Generic, "probe");
    let _ = provider.report_incoming(call, |_| {});
    let _ = provider.end_call(call_id, CallEndReason::RemoteEnded);
    provider.invalidate();
}

#[cfg(not(target_os = "ios"))]
fn main() {}
