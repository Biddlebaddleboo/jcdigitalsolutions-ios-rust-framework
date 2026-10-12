#[cfg(target_os = "ios")]
fn main() {
    use ios_call_provider::{CallProvider, CallProviderError, IncomingCall};
    use ios_voip_pushkit::{IncomingVoipPush, IosVoipPushService, PushKitEvent};
    use objc2::MainThreadMarker;

    fn report_after_payload_parse(
        push: IncomingVoipPush,
        provider: &CallProvider,
        call: IncomingCall,
    ) -> Result<(), CallProviderError> {
        push.report_incoming(provider, call, |_| {})
    }

    #[used]
    #[unsafe(no_mangle)]
    pub static IOS_VOIP_PUSHKIT_CALLKIT_LINK_PROBE: fn(
        IncomingVoipPush,
        &CallProvider,
        IncomingCall,
    ) -> Result<(), CallProviderError> = report_after_payload_parse;

    let Some(main_thread) = MainThreadMarker::new() else {
        return;
    };
    let _ = std::hint::black_box(&IOS_VOIP_PUSHKIT_CALLKIT_LINK_PROBE);
    let Ok(mut service) = IosVoipPushService::register(main_thread, |event| match event {
        PushKitEvent::TokenUpdated(token) => {
            let _ = token.as_bytes();
        }
        PushKitEvent::TokenInvalidated => {}
        PushKitEvent::Incoming(incoming) => {
            let _ = incoming.payload_json();
            drop(incoming);
        }
        _ => {}
    }) else {
        return;
    };
    service.invalidate();
}

#[cfg(not(target_os = "ios"))]
fn main() {}
