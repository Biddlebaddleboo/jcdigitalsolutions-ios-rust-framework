# Framework capability status

> Generated from `docs/capabilities/capability-status.json` by `ios-rust-validate generate-reports`. Do not edit manually.

## Coverage summary

| Metric | Count |
| --- | ---: |
| Capability families | 16 |
| Tracked capabilities | 114 |
| Implemented portable contracts | 36 |
| Partial portable contracts | 22 |
| Partial iOS implementations | 98 |

## Capability inventory

| ID | Family | Capability | Portable contract | iOS state | Parity |
| --- | --- | --- | --- | --- | --- |
| 001-core-app-ui-application-lifecycle | Core app/UI | application lifecycle | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 002-core-app-ui-windows-screens | Core app/UI | windows/screens | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 003-core-app-ui-basic-native-views-controllers-controls | Core app/UI | native controls, enabled state, system text styles/colors/font traits | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 004-core-app-ui-events-target-action-delegates | Core app/UI | events/target-action/delegates | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 005-core-app-ui-native-presentation | Core app/UI | native presentation | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 006-core-app-ui-pasteboard-clipboard | Core app/UI | pasteboard/clipboard | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 007-core-app-ui-share-system-composition | Core app/UI | share/system composition | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 008-core-app-ui-accessibility-extensions | Core app/UI | accessibility extensions | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 009-core-app-ui-native-escape-handles | Core app/UI | native escape handles | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 010-files-data-preferences-sandbox-files-directories | Files/data/preferences | sandbox files/directories | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 011-files-data-preferences-semantic-app-directories | Files/data/preferences | semantic app directories | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 012-files-data-preferences-preferences | Files/data/preferences | preferences | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 013-files-data-preferences-bundle-resources | Files/data/preferences | bundle/resources | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 014-files-data-preferences-file-coordination-provider-aware-operations-as-platform-extensions | Files/data/preferences | file coordination/provider-aware operations as platform extensions | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 015-files-data-preferences-url-uri-value-handling | Files/data/preferences | URL/URI value handling | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 016-files-data-preferences-data-byte-string-conversion-with-explicit-copy-behavior | Files/data/preferences | data/byte/string conversion with explicit copy behavior | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 017-security-auth-keychain-secure-storage | Security/auth | Keychain secure storage | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 018-security-auth-software-crypto-facade-with-system-hardware-key-escape | Security/auth | software crypto facade with system/hardware key escape | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 019-security-auth-seckey-secure-enclave | Security/auth | SecKey/Secure Enclave | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 020-security-auth-biometrics | Security/auth | biometrics | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 021-security-auth-passkeys-sign-in-with-apple | Security/auth | passkeys/Sign in with Apple | not_implemented | partial | not_tested_live_credential_query_or_Apple_parity |
| 022-security-auth-app-attest-devicecheck | Security/auth | App Attest/DeviceCheck | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 023-security-auth-privacy-authorization-status | Security/auth | privacy authorization status | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 024-networking-web-request-response-model | Networking/web | request/response model | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 025-networking-web-foreground-http | Networking/web | foreground HTTP | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 026-networking-web-background-transfer | Networking/web | background transfer | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 027-networking-web-low-level-network-connection-listener-where-appropriate | Networking/web | low-level network connection/listener where appropriate | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 028-networking-web-reachability-semantics-only-via-current-supported-apis | Networking/web | reachability semantics only via current supported APIs | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 029-networking-web-webkit | Networking/web | WebKit | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 030-networking-web-safari-system-browser-presentation | Networking/web | Safari/system browser presentation | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 031-notifications-background-local-push-notification-client-apis | Notifications/background | local/push notification client APIs | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 032-notifications-background-notification-response-delegate | Notifications/background | notification response/delegate | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 033-notifications-background-pushkit | Notifications/background | PushKit | not_implemented | partial | not_tested_live_APNs_or_device_behavior |
| 034-notifications-background-backgroundtasks | Notifications/background | BackgroundTasks | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 035-notifications-background-uikit-background-execution | Notifications/background | UIKit background execution | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 036-notifications-background-backgroundassets-where-applicable | Notifications/background | BackgroundAssets where applicable | not_implemented | partial | not_tested_live_queue_or_Apple_parity |
| 037-sensors-connectivity-location-geofencing-significant-change-where-supported | Sensors/connectivity | location/geofencing/significant-change where supported | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 038-sensors-connectivity-motion | Sensors/connectivity | motion | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 039-sensors-connectivity-bluetooth-central-peripheral-surfaces-required-by-common-apps | Sensors/connectivity | Bluetooth central/peripheral surfaces required by common apps | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 040-sensors-connectivity-nfc-session-tag-operations | Sensors/connectivity | NFC session/tag operations | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 041-sensors-connectivity-nearby-interaction | Sensors/connectivity | Nearby Interaction | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 042-sensors-connectivity-sensorkit | Sensors/connectivity | SensorKit | not_implemented | unsupported | not_tested_no_iOS_implementation |
| 043-sensors-connectivity-externalaccessory | Sensors/connectivity | ExternalAccessory | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 044-sensors-connectivity-accessorysetupkit | Sensors/connectivity | AccessorySetupKit | not_implemented | partial | not_tested_no_runtime_accessory_query |
| 045-sensors-connectivity-thread-network-system-pieces | Sensors/connectivity | Thread network/system pieces | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 046-camera-audio-media-camera-capture-session-device | Camera/audio/media | camera capture/session/device | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 047-camera-audio-media-microphone-audio-engine-i-o | Camera/audio/media | microphone/audio engine/I/O | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 048-camera-audio-media-playback | Camera/audio/media | playback | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 049-camera-audio-media-media-time-buffer-wrappers | Camera/audio/media | media time/buffer wrappers | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 050-camera-audio-media-video-encode-decode-through-videotoolbox | Camera/audio/media | video encode/decode through VideoToolbox | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 051-camera-audio-media-now-playing-media-control | Camera/audio/media | Now Playing/media control | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 052-camera-audio-media-replaykit | Camera/audio/media | ReplayKit | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 053-camera-audio-media-shazamkit-soundanalysis | Camera/audio/media | ShazamKit/SoundAnalysis | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 054-graphics-gpu-geometry-value-types-where-portable | Graphics/GPU | geometry/value types where portable | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 055-graphics-gpu-coregraphics-quartzcore-platform-operations | Graphics/GPU | CoreGraphics/QuartzCore platform operations | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 056-graphics-gpu-text-layout-platform-extension | Graphics/GPU | text layout platform extension | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 057-graphics-gpu-coreimage-imageio | Graphics/GPU | CoreImage/ImageIO | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 058-graphics-gpu-metal-metalkit | Graphics/GPU | Metal/MetalKit | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 059-graphics-gpu-mps-mpsgraph | Graphics/GPU | MPS/MPSGraph | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 060-graphics-gpu-accelerate-access | Graphics/GPU | Accelerate access | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 061-graphics-gpu-modelio | Graphics/GPU | ModelIO | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 062-graphics-gpu-scenekit-spritekit-platform-extension | Graphics/GPU | SceneKit/SpriteKit platform extension | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 063-ml-vision-language-core-ml-generic-inference | ML/vision/language | Core ML generic inference | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 064-ml-vision-language-vision | ML/vision/language | Vision | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 065-ml-vision-language-naturallanguage-system-model-features | ML/vision/language | NaturalLanguage system model features | not_implemented | partial | not_tested_live_asset_state_or_Apple_parity |
| 066-ml-vision-language-speech | ML/vision/language | Speech | not_implemented | partial | not_tested_live_authorization_or_Apple_parity |
| 067-ml-vision-language-foundation-models-residual-through-c-where-available | ML/vision/language | Foundation Models residual through C where available | not_implemented | partial | not_tested_no_runtime_or_Apple_parity |
| 068-personal-data-system-stores-photos | Personal data/system stores | Photos | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 069-personal-data-system-stores-contacts | Personal data/system stores | Contacts | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 070-personal-data-system-stores-eventkit | Personal data/system stores | EventKit | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 071-personal-data-system-stores-healthkit | Personal data/system stores | HealthKit | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 072-personal-data-system-stores-homekit | Personal data/system stores | HomeKit | not_implemented | partial | not_tested_live_HomeKit_behavior_or_Apple_parity |
| 073-personal-data-system-stores-safetykit | Personal data/system stores | SafetyKit | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 074-personal-data-system-stores-screentime-familycontrols-base-authorization | Personal data/system stores | ScreenTime/FamilyControls base authorization | not_implemented | partial | not_tested_live_authorization_or_Apple_parity |
| 075-personal-data-system-stores-deviceactivity-managedsettings-where-residual-support-exists | Personal data/system stores | DeviceActivity/ManagedSettings where residual support exists | not_implemented | unsupported | not_tested_no_iOS_implementation |
| 076-cloud-accounts-communication-cloudkit | Cloud/accounts/communication | CloudKit | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 077-cloud-accounts-communication-icloud-related-state | Cloud/accounts/communication | iCloud-related state | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 078-cloud-accounts-communication-watchconnectivity | Cloud/accounts/communication | WatchConnectivity | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 079-cloud-accounts-communication-gamekit | Cloud/accounts/communication | GameKit | implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 080-cloud-accounts-communication-classkit | Cloud/accounts/communication | ClassKit | not_implemented | partial | not_tested_live_activity_or_Apple_parity |
| 081-cloud-accounts-communication-messageui | Cloud/accounts/communication | MessageUI | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 082-cloud-accounts-communication-sharedwithyou | Cloud/accounts/communication | SharedWithYou | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 083-cloud-accounts-communication-callkit | Cloud/accounts/communication | CallKit | not_implemented | partial | not_tested_live_call_observation_or_Apple_parity |
| 084-cloud-accounts-communication-pushtotalk | Cloud/accounts/communication | PushToTalk | not_implemented | unsupported | not_tested_no_PushToTalk_channel_or_audio_implementation |
| 085-cloud-accounts-communication-carplay | Cloud/accounts/communication | CarPlay | not_implemented | unsupported | not_tested_no_entitled_CarPlay_scene_or_session |
| 086-commerce-services-storekit-2-via-c | Commerce/services | StoreKit 2 via C | not_implemented | partial | not_tested_live_purchase_status_or_Apple_parity |
| 087-commerce-services-legacy-storekit-only-where-compatibility-requires-it | Commerce/services | legacy StoreKit only where compatibility requires it | not_implemented | partial | not_tested_live_purchase_status_or_Apple_parity |
| 088-commerce-services-apple-pay-passkit | Commerce/services | Apple Pay/PassKit | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 089-commerce-services-adattributionkit-adservices-as-applicable | Commerce/services | AdAttributionKit/AdServices as applicable | not_implemented | partial | not_tested_no_runtime_or_Apple_parity |
| 090-commerce-services-apple-music-using-rest-mediaplayer-first-swift-residual-only-where-required | Commerce/services | Apple Music using REST/MediaPlayer first, Swift residual only where required | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 091-commerce-services-weatherkit-through-rest-native-http-where-appropriate | Commerce/services | WeatherKit through REST/native HTTP where appropriate | not_implemented | unsupported | not_tested_no_native_or_REST_WeatherKit_implementation |
| 092-commerce-services-tipkit-only-if-system-tipkit-behavior-is-specifically-requested-otherwise-framework-owned-tip-logic-may-be-portable | Commerce/services | TipKit only if system TipKit behavior is specifically requested; otherwise framework-owned tip logic may be portable | not_implemented | unsupported | not_tested_no_iOS_implementation |
| 093-maps-ar-spatial-mapkit | Maps/AR/spatial | MapKit | partial | partial | not_tested_live_map_geometry_markers_routes_or_user_tracking |
| 094-maps-ar-spatial-arkit | Maps/AR/spatial | ARKit | partial | partial | not_tested_live_behavior_or_Apple_parity |
| 095-maps-ar-spatial-realitykit-native-swift-residual-coverage-that-is-realistically-supportable | Maps/AR/spatial | RealityKit native/Swift residual coverage that is realistically supportable | not_implemented | partial | not_tested_no_photogrammetry_session_or_RealityKit_scene_parity |
| 096-maps-ar-spatial-roomplan | Maps/AR/spatial | RoomPlan | implemented | partial | not_tested_physical_device_or_live_scan |
| 097-maps-ar-spatial-dockkit | Maps/AR/spatial | DockKit | not_implemented | partial | not_tested_no_runtime_or_Apple_parity |
| 098-extension-entitlement-capabilities-networkextension-vpn | Extension/entitlement capabilities | NetworkExtension/VPN | implemented | partial | not_tested_live_profile_or_Apple_parity |
| 099-extension-entitlement-capabilities-fileprovider | Extension/entitlement capabilities | FileProvider | not_implemented | partial | not_tested_live_provider_registered_domains_or_count |
| 100-extension-entitlement-capabilities-extensionkit-foundation-where-feasible | Extension/entitlement capabilities | ExtensionKit/Foundation where feasible | not_implemented | unsupported | not_tested_no_iOS_implementation |
| 101-extension-entitlement-capabilities-browserenginekit-for-entitled-apps | Extension/entitlement capabilities | BrowserEngineKit for entitled apps | not_implemented | unsupported | not_tested_no_iOS_implementation |
| 102-extension-entitlement-capabilities-contactprovider | Extension/entitlement capabilities | ContactProvider | not_implemented | unsupported | not_tested_no_iOS_implementation |
| 103-extension-entitlement-capabilities-managedapp-distribution | Extension/entitlement capabilities | ManagedApp/Distribution | not_implemented | unsupported | not_tested_managed_device_or_distribution_flow |
| 104-extension-entitlement-capabilities-marketplacekit | Extension/entitlement capabilities | MarketplaceKit | not_implemented | unsupported | not_tested_no_iOS_implementation |
| 105-extension-entitlement-capabilities-mattersupport | Extension/entitlement capabilities | MatterSupport | not_implemented | partial | not_tested_no_runtime_or_Apple_parity |
| 106-extension-entitlement-capabilities-secureelementcredential | Extension/entitlement capabilities | SecureElementCredential | not_implemented | unsupported | not_tested_no_iOS_implementation |
| 107-extension-entitlement-capabilities-carkey | Extension/entitlement capabilities | CarKey | not_implemented | unsupported | not_tested_no_iOS_implementation |
| 108-extension-entitlement-capabilities-proximityreader | Extension/entitlement capabilities | ProximityReader | not_implemented | partial | not_tested_live_device_value_or_payment_readiness |
| 109-extension-entitlement-capabilities-lockedcameracapture | Extension/entitlement capabilities | LockedCameraCapture | not_implemented | unsupported | not_tested_no_iOS_implementation |
| 110-compiler-build-host-capabilities-app-intents-use-result-of-c-stage-0-1-if-unsupported-expose-no-fake-runtime-implementation | Compiler/build-host capabilities | App Intents: use result of C Stage 0/1; if unsupported, expose no fake runtime implementation. | not_implemented | unsupported | not_tested_no_iOS_implementation |
| 111-compiler-build-host-capabilities-widgetkit-support-management-data-logic-available-through-proven-interfaces-do-not-implement-a-swiftui-clone-merely-to-claim-full-rendering-support | Compiler/build-host capabilities | WidgetKit: support management/data logic available through proven interfaces; do not implement a SwiftUI clone merely to claim full rendering support. | not_implemented | partial | not_tested_no_runtime_or_Apple_parity |
| 112-compiler-build-host-capabilities-activitykit-support-if-layer-2-abi-work-is-proven | Compiler/build-host capabilities | ActivityKit: support if Layer-2 ABI work is proven. | not_implemented | partial | not_tested_live_behavior_or_Apple_parity |
| 113-compiler-build-host-capabilities-extension-bundle-metadata-helpers-only-through-supported-xcode-public-mechanisms | Extension/entitlement capabilities | extension bundle runtime metadata: one caller-selected extension-point identifier. | not_implemented | partial | not_tested_metadata_read_only |
| 114-capabilities-alarmkit-authorization-state-snapshot | Alarms and timers | AlarmKit current-app authorization-state snapshot | not_implemented | partial | not_tested_no_alarm_runtime_query_or_Apple_parity |


> Compilation, linkage and declarations are not evidence of live iPhone behavior.