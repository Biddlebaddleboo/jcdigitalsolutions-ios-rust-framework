# Registered validation profiles

> Generated from `tools/validation/specs/validation-v1.json`. Do not edit manually.

The following are configured validation profiles, not claims that every gate passed.

| Profile | Package | iOS targets | ABI scripts | Adapters | Offline |
| --- | --- | ---: | ---: | ---: | --- |
| ios-datagram | ios-datagram | 2 | 1 | 0 | True |
| ios-app-lifecycle | ios-app-lifecycle | 2 | 1 | 1 | False |
| ios-microphone-pcm-recorder | ios-microphone-recorder | 2 | 1 | 0 | False |
| ios-microphone-pcm-stream | ios-microphone-pcm-stream | 2 | 1 | 0 | False |
| ios-app-scenes | ios-app-scenes | 2 | 1 | 0 | False |
| ios-media-authorization | ios-media-authorization | 2 | 1 | 1 | False |
| ios-camera-capture-session | ios-camera-capture-session | 2 | 1 | 1 | False |
| ios-homekit-identify-status | ios-homekit-identify-status | 2 | 1 | 1 | False |
| ios-photogrammetry-status | ios-photogrammetry-status | 3 | 2 | 0 | False |
| ios-alarmkit-status | ios-alarmkit-status | 2 | 1 | 0 | True |
| ios-activitykit-status | ios-activitykit-status | 2 | 2 | 0 | False |
| ios-photos | ios-photos | 2 | 1 | 0 | False |
| ios-playback | ios-playback | 2 | 1 | 0 | False |
| ios-contacts-fetch | ios-contacts | 2 | 1 | 0 | False |
| ios-calendar | ios-calendar | 2 | 1 | 0 | False |
| ios-location-updates | ios-location | 2 | 1 | 0 | False |
| ios-ui | ios-ui | 2 | 3 | 0 | False |
| ios-sharing | ios-sharing | 2 | 1 | 0 | False |
| ios-notification-responses | ios-notification-responses | 2 | 1 | 1 | False |
| ios-call-provider | ios-call-provider | 2 | 1 | 1 | True |
| ios-notifications | ios-notifications | 2 | 1 | 0 | False |
| ios-navigation | ios-navigation | 2 | 1 | 0 | False |


## Inspect actual gates

```sh
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --list
ios-rust-validate --workspace-root "$PWD" --spec "$PWD/tools/validation/specs/validation-v1.json" --explain ios-app-lifecycle
```

A verified newer validator runs host package tests as registered gates; this inventory does not certify test completion.