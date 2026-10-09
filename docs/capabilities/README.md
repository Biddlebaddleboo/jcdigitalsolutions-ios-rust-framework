# Capability support

[`capability-status.json`](capability-status.json) is the canonical machine-readable and
human-readable support manifest. It has 114 capability rows across 16 families. Each row names a
crate/module, portability class, iOS implementation class, verified platform metadata, parity and
performance status, and native-escape status.

| iOS class | Meaning |
| --- | --- |
| `R` | Portable Rust implementation selected after correctness and performance evidence |
| `M` | Rust semantics/state over a minimal public Apple primitive |
| `B` | Apple/system-owned backend reached through Rust |
| `A` | Apple implementation preferred for performance or hardware reasons |
| `C` | Compiler/build/discovery contract |
| `X` | No supported implementation in this workstream; reason is in the row |

Current counts: 98 rows have `B` support, and all remain partial at the whole-capability
level pending runtime/parity evidence where applicable: two partial UIKit example rows, two partial
reusable UI-control rows, one partial native-escape row, one partial finite-Frame geometry row, one
partial CoreGraphics frame-intersection row, one partial CoreText system-font-metrics row,
three partial B1 sandbox-file and preference rows, and one partial row each for B2 Keychain, B3
request/response model, foreground HTTP, B4 local notifications, B5 current location, B6 plain-text clipboard, B7 outgoing
share, B8 accessibility, B9 acknowledgement alert, B10 packaged resources, B11 external HTTPS URL
handler, B12 local notification response, B13 background transfer, B15 motion, B16 local
authentication, B17 file coordination, B18 informational network-path snapshot, B19 raw-byte
CFData copies, B20 strict Foundation URL values, B21 CoreGraphics frame intersection, B22 finite
CoreMedia media time, B23 CoreText system-font metrics, B24 outbound TLS-over-TCP, B25 app-refresh
scheduling, B26 ImageIO metadata, B27 Photos authorization, B28 UIKit background execution, B29
Contacts authorization, B30 Calendar authorization, B31 HealthKit authorization, B32 Bluetooth
authorization, B33 WebKit navigation, B34 Bluetooth discovery, B35 iCloud Drive identity
presence, B36 camera/microphone authorization status, B37 NFC reader support, B39 one Nearby
Interaction device-capability query, B40 App Tracking Transparency status, B41 Metal
default-device presence, B42 DeviceCheck/App Attest API support, B43 Watch Connectivity session
support, B44 app-visible ExternalAccessory list presence, B45 legacy ReplayKit availability, B46
built-in SoundAnalysis classifier recognition, B47 other-app audio playback status, B48 HDR playback
eligibility, B49 MessageUI mail/text and SharedWithYou status, B50 VideoToolbox hardware-decode support,
B51 Apple Pay capability status, B52 CloudKit account-status snapshot, B53 SafetyKit Crash Detection
availability, B54 ARKit world-tracking support, B55 Game Center local-player status, B56 Core ML
compute-device availability, B57 Vision text-recognition revision support, B58 Speech authorization
status, B59 English Natural Language asset status, B60 legacy StoreKit purchase-ability status, B61 RoomPlan device-support status, B62 default camera video-device status, B63 StoreKit 2 purchase-ability status, B64 SafariServices HTTPS presentation, B65 Accelerate vDSP vector addition, B66 CommonCrypto SHA-256 only, B67 ModelIO extension support only, B68 MPS preferred-device presence only, B69 P-256 verification suitability only, B70 SpriteKit node position only, B71 MediaPlayer authorization status only, B72 CallKit call snapshot only, B73 MapKit geometry only, and B74 ClassKit deep-link marker only, B75 FileProvider registered-domain presence only, B76 ProximityReader device-model support only, B77 extension-point metadata read only, and B78 Sign in with Apple credential-state query only, B79 Personal VPN profile-status query only, B80 Family Controls raw authorization-status snapshot only, B81 UIKit scene activation-state counts only, B82 FileProvider registered-domain count only, B84 UIPasteboard string-presence preflight only, B85 opaque plain-bookmark creation/resolution without implicit security scope, B86 raw signed local-notification authorization status only, B87 location accuracy-authorization snapshot only, B88 raw notification alert/sound/badge setting snapshot only, B90 iOS-only regular-file byte length under the sandbox `AppPath`, and B91 additional raw local-notification settings, B92 UIKit Adjustable accessibility trait only, B93 iOS-only FileKind lookup for one AppPath, B94 raw CarPlay and optional Siri-announcement settings, and B95 UIKit NotEnabled accessibility metadata only, and B96 iOS-only no-follow POSIX modification-time snapshot, B97 unmanaged Background Assets queue count only, B98 UIKit KeyboardKey metadata only, B99 no-follow `(st_dev, st_ino)` near-time comparison only, B100 UIKit UpdatesFrequently metadata only, and B102 UIKit PlaysSound metadata only, B103 raw iOS POSIX permission/special bits only, B104 UIKit CausesPageTurn metadata only, B105 regular-file hard-link count only, B106 UIKit StartsMediaSession metadata only, and B107 no-follow POSIX status-change time only, B108 UIKit AllowsDirectInteraction metadata only, B109 regular-file allocated-block units only, B110 caller-supplied Background Assets manifest entry count only, B111 UIKit SummaryElement metadata only, B112 no-follow file access-time snapshot only, B113 owned manifest metadata only, B114 UIKit accessibility-element property read only, B115 raw BSD file-flag snapshot only, B117 accessibility text-property presence only, and B118 raw numeric file owner/group IDs only, B128 direct directory-entry count only, B131 directory-empty preflight only, B124 essential queue-entry count only, B127 non-default-priority queue-entry count only, B134 directory-entry kind counts only, B137 volume-capacity snapshot only, B120 known accessibility-trait membership only, and B123 child accessibility-elements-hidden state only, B126 Accessibility group-child property only, B129 Accessibility modal-child property only, B132 Accessibility language metadata only, and B135 Accessibility interaction-response metadata only, and B138 Accessibility navigation-style metadata only, and B141 Accessibility textual-context metadata only, and B144/B379 Accessibility container-type metadata only (iOS 13+ guard for SemanticGroup), B382 typed image-size adjustment for UIImageView/UIButton, B410 main-thread Guided Access restriction-state query only, and B434 main-thread Button Shapes setting snapshot only, B444 Large Content Viewer title metadata, B447 effective view layout direction, and B450 resolved interface-style snapshots only, B180 UIKit TabBar trait only, B186 expanded accessibility-state metadata only, B200 loaded Personal VPN configuration flags only, B206 Thread preferred-network availability only, B209 ActivityKit start-eligibility only and B233 MatterSupport request-API support only and B236 Foundation Models default-model readiness only and B239 AdAttributionKit app-impression support only and B245 DockKit system-tracking setting only and B254 WidgetKit timeline-reload requests only, B274 AccessorySetupKit previously-selected accessory count only, B280 AlarmKit authorization-state snapshot only, and B291/B298 Photogrammetry hardware-support and input-limit snapshots only, and B339/B394/B445/B449/B452 HomeKit per-accessory identify/profile count, host-supplied category metadata, and setup-result identifier count only; 16 rows are `X` for iOS runtime support. Counts are per capability row, so B3's shared `ios-network` backend appears for both the
request/response model and foreground HTTP. All 16 remaining `X` rows name a specific scope or toolchain gap; no generic missing-facade/backend reason remains. `X` is current
workstream status, not proof that Rust cannot call an Apple API. D90–D99 audits record scoped outcomes for rows 104–113; row 104 and rows 106–110 remain `X`, row 105 has B233's request-API support partial, row 112 has B209's ActivityKit start-eligibility partial, row 111 has B254's timeline-reload request partial, and row 113 has B77's runtime metadata partial. Row 114 is B280's AlarmKit authorization-state snapshot; see the linked [D90](../../PLAN_CAPABILITIES_MARKETPLACEKIT.md), [D91](../../PLAN_CAPABILITIES_MATTERSUPPORT.md), [D92](../../PLAN_CAPABILITIES_SECURE_ELEMENT_CREDENTIAL.md), [D93](../../PLAN_CAPABILITIES_CARKEY.md), [D94](../../PLAN_CAPABILITIES_PROXIMITYREADER.md), and [D95](../../PLAN_CAPABILITIES_LOCKED_CAMERA_CAPTURE.md), [D96](../../PLAN_CAPABILITIES_EXTENSION_BUNDLE_METADATA.md), [D97](../../PLAN_CAPABILITIES_WIDGETKIT.md), [D98](../../PLAN_CAPABILITIES_ACTIVITYKIT.md), and [D99](../../PLAN_CAPABILITIES_APP_INTENTS.md) audits. HomeKit row 072 remains partial for B339's iOS 11.3+ host-supplied `HMAccessory.supportsIdentify` snapshot, B394's retained profiles-array length, and B445/B449's host-supplied category identifier and localized display text, and B452's count of identifiers in a host-supplied successful setup result. These reads do not create `HMHomeManager`, request permission, enumerate accessories, or report authorization, reachability, or readiness. The [B173/B322 audits](../../PLAN_CAPABILITIES_HOMEKIT.md) still exclude prompt-free authorization and reachability snapshots, and B373 excludes a `nonatomic` `isBlocked` getter without a documented thread-safety contract; the host owns `com.apple.developer.homekit`, `NSHomeKitUsageDescription`, prior authorization, and each live accessory/category object's source and lifetime ([B339/B394/B445/B449 implementation notes](../../PLAN_CAPABILITIES_HOMEKIT.md)). AccessorySetupKit row 044 is partial only for B274's asynchronous count of previously selected accessories after `ASAccessorySession` activation. It does not present a picker, discover nearby accessories, expose identities, or claim proximity, connection, transport availability, or readiness. The strongly linked binding requires an iOS/iPadOS 18.0+ host deployment target; host Info.plist configuration remains app-owned ([B274 plan](../../PLAN_CAPABILITIES_ACCESSORY_SETUP.md)).
Row 021 is partial (`B`) only for B78's prior-user credential-state query under a conservative Sign in with Apple entitlement requirement. Passkeys, general sign-in readiness, and the original non-entitled scope remain unsupported ([D66 audit and B78 follow-up](../../PLAN_CAPABILITIES_PASSKEYS.md)).
Row 074 ScreenTime/FamilyControls has B80 partial support for a raw signed authorization-status
snapshot through a compiler-matched C `swiftcall` bridge. The unsafe call requires the main dispatch
queue. It does not prove entitlement presence or approval, activity-data access, or control use; Apple
does not document query-only entitlement semantics ([D67/B80 plan](../../PLAN_CAPABILITIES_FAMILY_CONTROLS_STATUS.md)).
Row 010 has B296 partial support for iOS-only `IosFiles::entry_object_kind(AppPath)`, a no-follow `fstatat` snapshot that distinguishes regular files, directories, symlinks, FIFOs, sockets, block devices, and character devices while preserving unknown mode bits. It does not open/follow the final entry, change portable `FileKind`, or guarantee a later operation; host use requires an applicable approved File Timestamp reason in `PrivacyInfo.xcprivacy` ([B296 plan](../../PLAN_IOS_ENTRY_OBJECT_KIND.md)). B301 adds `IosFiles::entry_stored_backup_time(AppPath)` for a regular file or directory through descriptor-bound `ATTR_CMN_BKUPTIME`; this is only a filesystem-stored marker, not proof of OS/iCloud backup completion or inclusion, and host use requires an applicable approved File Timestamp reason in `PrivacyInfo.xcprivacy` ([B301 plan](../../PLAN_IOS_FILE_BACKUP_TIME_MARKER.md)).
Row 001 has B81 partial support for point-in-time `connectedScenes` activation-state counts; it does not deliver lifecycle events or claim scene visibility ([B81 plan](../../PLAN_IOS_SCENE_SNAPSHOT.md)). Row 014 includes B83 unsafe resolution of caller-asserted plain bookmark data and B85 opaque creation/resolution that omits implicit security scope; neither grants access or adds user URLs to the sandbox `Files` facade. B90 adds an iOS-only regular-file byte-length snapshot under a validated sandbox `AppPath` without reading content or changing the portable file contract; B128 counts direct directory names across all entry kinds and byte names without a Rust-owned listing or per-entry kind lookups, and B131 adds an early-exit emptiness query. Both are best-effort under concurrent mutation, and neither is a deletion guard; B134 adds fixed-width regular-file, directory, and `Other` totals using no-follow per-name lookup, including non-UTF-8 names, and errors rather than return partial counts if a name vanishes; B137 reports only the volume-wide checked `f_bavail * f_bsize` value, not app quota or write certainty, and requires a host Disk Space privacy-manifest reason matching use; B146 reports checked total mounted-volume capacity, not physical-device capacity or app quota; B140 declines a cheap directory-count API because the Foundation key is optional and the descriptor count may be expensive, 32-bit, and unsupported by some volumes; B143 declines `f_bfree` because it includes reserved blocks the sandbox cannot consume, while B137 already reports `f_bavail`; B149 declines volume-wide free-node counts because they are not an app budget or a create-success guarantee ([B83 plan](../../PLAN_IOS_BOOKMARK_RESOLUTION.md), [B85 plan](../../PLAN_IOS_BOOKMARK_CREATION.md), [B90 plan](../../PLAN_IOS_FILE_SIZE.md), [B93 plan](../../PLAN_IOS_ENTRY_KIND.md), [B96 plan](../../PLAN_IOS_FILE_MODIFICATION_TIME.md), [B99 plan](../../PLAN_IOS_FILE_IDENTITY.md), [B101 audit](../../PLAN_IOS_FILE_CREATION_TIME.md), [B103 plan](../../PLAN_IOS_FILE_PERMISSION_BITS.md), [B105 plan](../../PLAN_IOS_FILE_LINK_COUNT.md), [B107 plan](../../PLAN_IOS_FILE_STATUS_CHANGE_TIME.md), [B109 plan](../../PLAN_IOS_FILE_ALLOCATED_BLOCKS.md), [B112 plan](../../PLAN_IOS_FILE_ACCESS_TIME.md), [B115 plan](../../PLAN_IOS_BSD_FILE_FLAGS.md), [B118 plan](../../PLAN_IOS_FILE_OWNER_IDS.md), [B128 plan](../../PLAN_IOS_DIRECTORY_ENTRY_COUNT.md), [B131 plan](../../PLAN_IOS_DIRECTORY_EMPTY_CHECK.md), [B134 plan](../../PLAN_IOS_DIRECTORY_KIND_COUNTS.md), [B137 plan](../../PLAN_IOS_VOLUME_CAPACITY.md), [B146 plan](../../PLAN_IOS_VOLUME_TOTAL_CAPACITY.md), [B140 audit](../../PLAN_IOS_FAST_DIRECTORY_COUNT.md), [B143 audit](../../PLAN_IOS_VOLUME_FREE_BLOCKS.md), [B149 audit](../../PLAN_IOS_VOLUME_FILE_NODE_COUNTS.md), [B121 audit](../../PLAN_IOS_FILE_IO_BLOCK_SIZE.md), [B122 audit](../../PLAN_IOS_FILE_SPECIAL_DEVICE_NUMBER.md), [B125 audit](../../PLAN_IOS_FILE_GENERATION_NUMBER.md)). B178 declines the ambiguous `NSURLVolumeMaximumFileSizeKey` `NSNumber` type, B181/B184/B187 decline volume hard-link/symlink/advisory-lock support values because the facade has no matching operations, and B190 declines a sparse-file support value without a sparse-file operation ([B178 audit](../../PLAN_IOS_VOLUME_MAXIMUM_FILE_SIZE.md), [B181 audit](../../PLAN_IOS_VOLUME_HARD_LINK_SUPPORT.md), [B184 audit](../../PLAN_IOS_VOLUME_SYMLINK_SUPPORT.md), [B187 audit](../../PLAN_IOS_VOLUME_ADVISORY_LOCKING.md)). B196 adds a regular-file copy-on-write clone without destination replacement; B199 adds a cached volume clone-support hint, not a per-operation guarantee. B202 declines volume permission support without an fd-bound setter, and B205 declines backup exclusion because the public setter re-resolves a URL instead of using the validated descriptor. B208 adds a no-follow regular-file extended-flags snapshot that preserves raw bits but does not identify a specific clone peer or guarantee durable allocation; B211 adds an opaque clone ID for pure-clone comparison only; B214 adds the point-in-time full-clone count, not partial peers, IDs, or paths, and needs an applicable approved File Timestamp host reason in PrivacyInfo.xcprivacy; B220 declines recursive directory generation count because no app-callable APFS dir-stats mark operation exists; B223 adds ATTR_CMNEXT_PRIVATESIZE as bytes not trapped in clone/snapshot data, not allocated size or future-space assurance; B226 adds ATTR_CMNEXT_LINKID for current-mounted-volume comparison only, with no cross-mount or content-identity claim; B231 declines mount-relative `ATTR_CMNEXT_RELPATH` due hard-link ambiguity and no facade operation; B232 declines `ATTR_CMNEXT_ATTRIBUTION_TAG` because no public bundle-ID mapping or setter exists; B235 reports `ATTR_DIR_ALLOCSIZE` for the directory object only, not its descendants; B237/B238/B240/B241 decline directory logical size, directory I/O block size, directory link count, and mount status because no supported app-data operation uses those fields; B242 adds a no-follow total fork-size snapshot for one regular file, including logical bytes in all forks but not fork content or physical allocation; B244 reports data-fork allocated bytes only, excluding resource-fork allocation; B248 reports resource-fork allocated bytes only, and B251 reports resource-fork logical length only; neither exposes fork contents or infers absence from zero; B247 declines a raw fork count because no facade operation uses it; B253 declines a duplicate all-fork allocation value because B109/B244/B248 already cover allocated-block and fork-specific values ([B242 plan](../../PLAN_IOS_FILE_TOTAL_FORK_SIZE.md), [B244 plan](../../PLAN_IOS_FILE_DATA_FORK_ALLOCATED_SIZE.md)) ([B196](../../PLAN_IOS_FILE_CLONING.md), [B199](../../PLAN_IOS_VOLUME_CLONING_SUPPORT.md), [B202](../../PLAN_IOS_VOLUME_ACCESS_PERMISSION_SUPPORT.md), [B205](../../PLAN_IOS_BACKUP_EXCLUSION.md), [B208](../../PLAN_IOS_FILE_EXTENDED_FLAGS.md), [B211](../../PLAN_IOS_FILE_CLONE_ID.md), [B214](../../PLAN_IOS_FILE_FULL_CLONE_COUNT.md), [B220](../../PLAN_IOS_RECURSIVE_DIRECTORY_GENERATION.md), [B223](../../PLAN_IOS_FILE_PRIVATE_SIZE.md), [B226](../../PLAN_IOS_FILE_LINK_ID.md), [B231](../../PLAN_IOS_MOUNT_RELATIVE_FILE_PATH.md), [B232](../../PLAN_IOS_FILE_ATTRIBUTION_TAG.md)). Row 031 adds B86 raw signed authorization-status values, including provisional, ephemeral, and unknown values, plus B88 raw alert, sound, and badge setting values and B91 Notification Center/Lock Screen plus optional critical-alert/time-sensitive/scheduled-delivery values through prompt-free settings queries; unknown signed setting values are preserved, and none of these snapshots changes D3 normalization or proves delivery readiness ([B86 plan](../../PLAN_IOS_NOTIFICATIONS.md), [B88 plan](../../PLAN_IOS_NOTIFICATION_SETTINGS.md), [B91 plan](../../PLAN_IOS_NOTIFICATION_SETTINGS_EXTENDED.md), [B94 plan](../../PLAN_IOS_NOTIFICATION_SETTINGS_SURFACES.md)). Row 037 adds B87's iOS 14+ prompt-free full/reduced location-accuracy authorization snapshot, preserving unknown signed values; it does not change portable D4 or implement geofencing, significant-change monitoring, or background location ([B87 plan](../../PLAN_IOS_LOCATION.md)). Row 008 adds B92 Adjustable, B95 NotEnabled, B98 KeyboardKey, B100 UpdatesFrequently, B102 PlaysSound, B104 CausesPageTurn, B106 StartsMediaSession, B108 AllowsDirectInteraction, and B111 SummaryElement traits to the existing accessibility adapter; B114 reads the current accessibility-element flag, and B117 checks only nullable label/hint/value presence, B120 checks membership for a known trait only, and B123 gets/sets whether child accessibility elements are hidden through an iOS 5.0+ selector guard that returns `AccessibilityApiUnavailable` when absent and preserves the iOS 4.0 package floor; B126 adds guarded group-children getters/setters for logical parent groups, and B129 adds guarded modal-child getters/setters for actually modal content. Neither claims assistive traversal or app/view presentation modality. B132 adds guarded BCP 47 accessibility-language metadata without validation/normalization or dynamic callback behavior; B135 adds guarded interaction-response metadata but no handler. B138 adds selector-guarded typed navigation style, maps unknown native values to `None`, and currently affects Switch Control only, not VoiceOver. B141 adds selector-guarded textual-context metadata from seven named UIKit constants; it does not invoke the iOS 17 callback or claim assistive output. B144 adds the guarded `Unspecified`/`List`/`Landmark` container-type subset; B379 adds `SemanticGroup` with a caller iOS 13+ guard, while `DataTable` remains excluded because it needs the data-table protocol. B147 declines direct-touch options because their VoiceOver passthrough/audio behavior needs a dedicated interaction contract. These are metadata only, with caller-owned behavior where required. B108 does not route direct touch or configure DirectTouchOptions; B111 does not control summary content or presentation timing; B114 reports only the element property, not visibility or focus; B117 returns only nullable label/hint/value presence, not text, and empty strings count as present. Row 036 has B97 partial support for a read-only count of unmanaged Background Assets queue entries at iOS 16.1+ and B110 support for an iOS 26.0+ count of entries in caller-supplied JSON plus B113 owned manifest identifier/size/version metadata and B116 optional userInfo JSON bytes; B119 found no further safe read-only manifest field in the installed SDK/binding, with language beta for iOS 27.0. B124 counts only returned queue entries marked essential at iOS 16.4+; it does not count installed assets or all essential assets on device. B127 counts only callback-returned queue entries whose priority differs from `BADownloaderPriorityDefault` at iOS 16.1+; neither count implies installation or general support. None reports local installation or validates host setup ([Background Assets plan](../../PLAN_CAPABILITIES_BACKGROUND_ASSETS.md)).
Row 014 also has B266 for `ATTR_CMN_DOCUMENT_ID` on a regular file: zero maps to `None`, an omitted attribute maps to `Unsupported`, and the value is not an inode, content hash, clone/link ID, or cross-volume identity. B359 separately reads stored `ATTR_CMN_CRTIME` only after checking the same descriptor’s volume support mask; this value is mutable metadata and does not use `st_birthtime`. B365 reads `ATTR_VOL_SPACEUSED` as volume-wide used bytes, which may differ from total minus free on space-sharing volumes and is not app usage or quota. Host use of these `fgetattrlist` snapshots requires an applicable approved File Timestamp reason in `PrivacyInfo.xcprivacy` ([B266 plan](../../PLAN_IOS_FILE_DOCUMENT_ID.md), [B359 plan](../../PLAN_IOS_FILE_CREATION_TIME_ATTRIBUTE.md), [B365 plan](../../PLAN_IOS_VOLUME_USED_CAPACITY.md)).
Row 008 adds B434 `button_shapes_are_enabled(&MainThread)` as an iOS 14.0+ caller-guarded snapshot of UIKit’s Button Shapes preference; the query is deprecated from iOS 26.1 in favor of `AXShowBordersEnabled`, and it makes no rendered-button claim ([accessibility plan](../../PLAN_IOS_ACCESSIBILITY.md)). B444 reads a supplied view’s Large Content Viewer title, B447 snapshots its effective layout direction for immediate content, and B450 snapshots its resolved interface style; none promises UI presentation, subtree inheritance, rendered color, or later change observation. Row 008 includes B255 Guided Access-gated AssistiveTouch state; B258 Guided Access, B261 Reduce Motion, B268 Reduce Transparency, and B271/B273 Speak Screen/Speak Selection snapshots and B276 Classic Invert use main-thread proof. B271/B273 require caller iOS 8+ availability checks and B276 requires iOS 6+; these snapshots do not invoke speech, observe change notifications, or claim assistive output ([accessibility plan](../../PLAN_IOS_ACCESSIBILITY.md)).
Row 067 Foundation Models is partial (`B`) for B236’s iOS 26.0+ `SystemLanguageModel.default.isAvailable` readiness Boolean only. The beta API does not expose unavailable reasons or support session creation, inference, generation, prompts, context size, supported languages, or use-case support; no runtime/model call was made ([D69/B236 plan](../../PLAN_CAPABILITIES_FOUNDATION_MODELS.md)).
Recent bounded follow-ups extend existing partial rows: Row 008 B153 adds selector-guarded iOS 5 accessibility activation-point get/set using screen-coordinate `CGPoint` values; callers must recompute after layout/screen-position changes, and the adapter does not guarantee assistive activation; B156 adds iOS 11 attributed-label set/presence operations, with UIKit copy and label coupling but no dynamic callback or speech claim ([accessibility plan](../../PLAN_IOS_ACCESSIBILITY.md)). B159 adds selector-guarded iOS 11 attributed-hint set/presence with UIKit copy and plain-hint coupling, but no dynamic callback or speech claim ([accessibility plan](../../PLAN_IOS_ACCESSIBILITY.md)). B162 adds selector-guarded iOS 11 attributed-value set/presence with UIKit copy and plain-value coupling, but no dynamic callback or speech claim ([accessibility plan](../../PLAN_IOS_ACCESSIBILITY.md)). B169 adds selector-guarded iOS 13 attributed user-input-label set/presence and preserves primary-first ordering without speech/recognition guarantees ([accessibility plan](../../PLAN_IOS_ACCESSIBILITY.md)). B171 adds selector-guarded iOS 7 screen-coordinate `accessibilityPath` set/presence; callers update geometry and no highlight/activation behavior is promised ([accessibility plan](../../PLAN_IOS_ACCESSIBILITY.md)). B174 adds selector-guarded screen-coordinate `accessibilityFrame` get/set using `CGRect`; it claims no conversion, layout, visibility, or assistive behavior ([accessibility plan](../../PLAN_IOS_ACCESSIBILITY.md)). B177 adds selector-guarded iOS 5 `accessibilityIdentifier` get/set for UI Automation only, with no uniqueness or assistive-output claim; B180 adds the iOS 10 `TabBar` trait for an ordered tab list with caller-owned `isAccessibilityElement` state, while B183 declines iOS 17 `ToggleButton` because the shared mapping cannot report per-trait availability; B186 adds iOS 18 selector-guarded expanded-state get/set without invoking the callback block or expanding content; B189 declines the untyped `accessibilityElements` child array because the borrowed-view adapter has no ownership, cleanup, or container-lifecycle contract; B192 adds a guarded per-view focus Boolean; B195 copies and sorts currently focused assistive-technology identifiers, preserving native nil; B198 declines the global focused-element function because its arbitrary retained result has no safe borrowed-view contract; B201 copies the user-input-label array with native nil/empty and order preserved; B204 copies the full accessibilityLanguage string without normalization; B207 copies the full textual-context string without normalization or callback invocation; B210/B213/B216 add selector-guarded nullable plain label/hint/value getters with nil/empty preservation and Rust-owned text; B219/B222 return retained immutable attributed label/hint values. B225 returns the retained immutable attributed-value property; B228 copies the attributed user-input-label array; B234 returns an owned accessibility-path copy; B243 preserves the raw accessibility-trait mask; B246 snapshots the iOS 17 direct-touch option mask only; B249 reads the main-thread VoiceOver-running Boolean; B252 reads the main-thread Switch Control-running Boolean with an iOS 8+ caller guard; B255 returns `None` when Guided Access is off, otherwise snapshots AssistiveTouch state; B279/B281/B284/B287/B290/B292/B295 add On/Off Labels, Bold Text, Mono Audio, Shake to Undo, Differentiate Without Color, Closed Captions + SDH, and cross-fade preference snapshots, all on the main thread and without observers or system behavior; B295 is true only when both Reduce Motion and Prefer Cross-Fade Transitions are enabled. B297 adds selector-guarded `accessibility_ignores_invert_colors()` get/set; its setter opts the entire view subtree out of accessibility-requested color inversion but does not predict colors or settings. B299 adds selector-guarded `shows_large_content_viewer()` get/set for iOS 13+; the property has no effect without the host attaching `UILargeContentViewerInteraction` and does not promise presentation. These add no observer or UI action. B305 adds `increase_contrast_is_enabled(&MainThread)` as a main-thread, iOS 8+ snapshot of UIKit’s Increase Contrast setting; it does not observe setting changes, mutate colors, or claim UIKit applies contrast to app-owned drawing. B309 adds `color_filters_or_grayscale_preference_is_enabled(&MainThread)` as a main-thread, iOS 8+ snapshot of UIKit’s Color Filters/Grayscale preference state; it does not identify the active filter or report rendered colors. B313 adds `video_autoplay_previews_are_enabled(&MainThread)` as a main-thread, iOS 13+ snapshot of UIKit’s Auto-Play Video Previews setting; it does not control or report playback behavior for app video content. B317 adds `hearing_device_paired_ear(&MainThread) -> HearingDevicePairingStatus` as an iOS 10+ ear-side pairing snapshot with unknown bits preserved; it does not expose device identity or report connection, streaming, or audio-route state. B325 adds `AccessibilityMetadata::convert_frame_to_screen_coordinates(frame_in_view: CGRect) -> CGRect`, mapping from the borrowed view’s coordinate space to screen coordinates for `accessibilityFrame`; it does not mutate the view or claim visibility or presentation. B330 adds `AccessibilityMetadata::convert_path_to_screen_coordinates(path_in_view: &UIBezierPath) -> Retained<UIBezierPath>`, returning a new screen-coordinate path from the borrowed view’s coordinate space; it leaves both inputs unchanged and makes no rendering, visibility, or presentation claim. B252 is a direct symbol without an availability guard; callers must ensure iOS 8+. These getters invoke no callback blocks or claim speech/output. None invokes dynamic callback blocks or claims speech/output ([accessibility plan](../../PLAN_IOS_ACCESSIBILITY.md)).  Row 008 B150 adds selector-guarded iOS 13 accessibility user-input-label metadata only, with no speech-match guarantee ([accessibility plan](../../PLAN_IOS_ACCESSIBILITY.md)). Row 014 B152 reports only the retained volume’s `MNT_RDONLY` mount bit, while B155 returns `f_iosize` as an informational sizing hint; neither establishes effective write access or guarantees I/O performance, and both require a host Disk Space privacy-manifest reason matching use ([B152 plan](../../PLAN_IOS_VOLUME_READ_ONLY.md), [B155 plan](../../PLAN_IOS_VOLUME_IO_SIZE.md)). B158 declines statfs identity fields without a stable app/container contract; B161 declines additional mount flags as effective path authorization; B164 reports only a direct-child filename-component limit, not total/nested path limits or later-operation success ([B158 audit](../../PLAN_IOS_VOLUME_FILESYSTEM_IDENTITY.md), [B161 audit](../../PLAN_IOS_VOLUME_MOUNT_RESTRICTIONS.md), [B164 plan](../../PLAN_IOS_APP_DIRECTORY_NAME_MAX.md)). B170 adds a no-follow current-effective-UID `ATTR_CMN_USERACCESS` snapshot for read/write/execute-search permission bits; it is not a guarantee that a later operation succeeds ([B170 plan](../../PLAN_IOS_ENTRY_EFFECTIVE_ACCESS.md)). B172 exposes cached Foundation `RENAME_EXCL`/`RENAME_SWAP` volume support as `Option<bool>`; the cache is created with `IosFiles::new` and does not guarantee a later rename ([B172 plan](../../PLAN_IOS_VOLUME_RENAME_SUPPORT.md)). B175 adds cached Foundation case-sensitive-name and case-preserved-name support as `Option<bool>` without changing `AppPath` comparison semantics ([B175 plan](../../PLAN_IOS_VOLUME_NAME_SUPPORT.md)). Row 036 B165 declines `_PC_PATH_MAX` because `openat` walks validated path components, and B168 declines file-protection-class exposure because no public numeric mapping exists and Foundation alternatives would re-resolve paths ([B165 audit](../../PLAN_IOS_PATH_MAX.md), [B168 audit](../../PLAN_IOS_FILE_PROTECTION_CLASS.md)). Row 036 B130/B133 copy optional general and essential download allowances from the system-supplied `BAAppExtensionInfo` callback object, and B136 classifies its typed `BAContentRequest` while preserving unknown raw values; these do not schedule downloads or report installed assets; B139 declines a typed `BAErrorDomain` decoder because the SDK domain symbol is iOS 17-only with no documented literal while this package supports iOS 16.1; B142 declines managed-pack status because the available query is deprecated/network-capable and `sharedManager` opts the host into managed mode. ([Background Assets plan](../../PLAN_CAPABILITIES_BACKGROUND_ASSETS.md)). B206 adds row 045's iOS 16.4+ preferred-network availability query as a narrow entitlement-scoped partial. It requires `com.apple.developer.networking.manage-thread-network-credentials` and Apple distribution approval; it does not report local radio, active connectivity, or border-router capability ([B206 plan](../../PLAN_CAPABILITIES_THREAD.md)). The capability-row count is 98 of 114 (`B`, 86.0%), with 16 `X`; runtime/parity and other plan acceptance gates remain outstanding.
B236 exposes only the default-model readiness Boolean through a compiler-derived Swift ABI bridge; sessions, inference, generation, and other Foundation Models APIs remain unsupported ([D69/B236 plan](../../PLAN_CAPABILITIES_FOUNDATION_MODELS.md)).
Row 075 DeviceActivity/ManagedSettings remains unsupported: B176 confirms that the public Objective-C authorization getter has no documented useful authorization meaning, prompt behavior, threading, or query-only entitlement semantics; B376/B383 confirm that the Swift-only schedule and event queries expose host-configured monitoring data rather than general framework capability, with Rust collection/value ABI and host-execution contracts still absent ([D71/B176/B376/B383 feasibility plan](../../PLAN_CAPABILITIES_DEVICE_ACTIVITY.md)).
Row 092 TipKit remains unsupported because `Tip.status` is a Swift-only per-tip eligibility value,
not global readiness or proof of presentation; no C/Objective-C entry point or generated Rust binding
is available; B191 confirms the API is only per-concrete-tip state and has no C/Objective-C entry ([D73/B191 feasibility plan](../../PLAN_CAPABILITIES_TIPKIT.md)).
Rows 033/036/042 remain unsupported for distinct host/service gates: PushKit's cached VoIP token is
not a readiness query and a real path needs APNs/delegate and call-handling lifecycle; BackgroundAssets
has no global readiness query and depends on host extension/configuration; SensorKit's only per-sensor status query uses deprecated `SRSensorReader`; its replacement `SRReader<Sensor>` remains beta and is absent from the installed SDK/binding, while the required research entitlement is enforced at app launch. See the [PushKit](../../PLAN_CAPABILITIES_PUSHKIT.md),
[BackgroundAssets](../../PLAN_CAPABILITIES_BACKGROUND_ASSETS.md), and [SensorKit](../../PLAN_CAPABILITIES_SENSORKIT.md)
feasibility plans.
Row 045 Thread is partial (`B`) for B206's Rust future over `THClient.isPreferredNetworkAvailableWithCompletion:` (iOS 16.4+). The host needs `com.apple.developer.networking.manage-thread-network-credentials` and Apple distribution approval. This reports only preferred-network availability, not local radio, active connectivity, or border-router capability; the Swift-only `MatterAddDeviceRequest.isSupported` query covers Matter setup, not Thread ([D75/B206 plan](../../PLAN_CAPABILITIES_THREAD.md)).
Row 112 ActivityKit is partial (`B`) for B209's iOS 16.1+ `ActivityAuthorizationInfo.areActivitiesEnabled` snapshot. It reports only whether this app can start a Live Activity at the query time; it does not create, update, end, or observe activities. Hosts that offer Live Activities must set `NSSupportsLiveActivities` and provide separate WidgetKit/SwiftUI presentation ([D98/B209 plan](../../PLAN_CAPABILITIES_ACTIVITYKIT.md)).
Row 097 DockKit is partial (`B`) for B245’s iOS 17+ `DockAccessoryManager.shared.isSystemTrackingEnabled` snapshot only. The physical-device path uses a compiler-derived weak `swiftcall` bridge and an owned manager; Simulator returns `NativeApiUnavailable`. It does not report accessory presence, active tracking, camera state, or operation success, and it does not expose accessory-change events ([D78/B197 audit and B245 implementation](../../PLAN_CAPABILITIES_DOCKKIT.md)).
Row 091 WeatherKit remains `X`: B250 confirms the REST availability endpoint gives coordinate/data-set availability only, not device or forecast readiness. A host-selected REST client still needs a trusted ES256 developer token, typed response model, and attribution contract; native WeatherKit remains Swift-only ([D79/B250 audit](../../PLAN_CAPABILITIES_WEATHERKIT.md)). Row 111 WidgetKit is partial (`B`) for B254's iOS 14+ `WidgetCenter.shared.reloadAllTimelines()` request only. It requests reload of configured widgets in the containing app but does not guarantee provider success, rendering, or timing; provider/timeline and SwiftUI APIs remain unsupported ([D97/B254 plan](../../PLAN_CAPABILITIES_WIDGETKIT.md)). Row 095 RealityKit is partial (`B`) only for B291's iOS 17.0+ `PhotogrammetrySession.isSupported` hardware-support snapshot. B298 also reads the two device-specific `PhotogrammetrySession.limits` values: maximum input dimension and image/sample count as exact signed `i64` values through metadata-sized/aligned resilient-value storage and value-witness destruction. Neither the hardware predicate nor limits create sessions, read images, perform reconstruction/capture/UI, guarantee success or quality, or add scene/entity/render support; the `ARView` shell remains insufficient for useful scene work ([D80/B194/B291/B298 plan](../../PLAN_CAPABILITIES_REALITYKIT.md)).
Row 114 AlarmKit is partial (`B`) only for B280's iOS 26.0+ `AlarmManager.shared.authorizationState` snapshot. Its C bridge follows compiler-derived `swiftcall` and resilient-enum value-witness evidence; it requests no authorization, schedules no alarm, reads no alarm content, and observes no updates. Evidence used Xcode 26.6 / iOS SDK 26.5, below the Xcode 27.x baseline; no app link or runtime query ran ([B280 plan](../../PLAN_CAPABILITIES_ALARMKIT.md)).
Row 098 NetworkExtension/VPN is partial (`B`) for B79's read-only caller-app profile-status snapshot and B200's loaded `isEnabled`/`isOnDemandEnabled` flags. The host requires `com.apple.developer.networking.vpn.api = ["allow-vpn"]`; a false enabled flag may reflect another profile, and the on-demand flag does not prove rule setup or automatic connection. No preference mutation, tunnel control, provider extensions, routes, reachability, or system-wide VPN state are covered ([D82 audit and B79/B200 plan](../../PLAN_CAPABILITIES_NETWORK_EXTENSION.md)).
Row 084 PushToTalk remains unsupported because no standalone support/authorization query exists and useful operation needs entitlement, background mode, microphone consent, APNs channel restoration, and audio lifecycle ([D83/B179 audit](../../PLAN_CAPABILITIES_PUSHTOTALK.md)). Row 085 CarPlay remains unsupported because its session-configuration values describe the connected vehicle, not device availability; B182 confirms `supportsVideoPlayback` is only a connected-session property. Useful access requires an approved category entitlement and host scene/session lifecycle ([D84/B182 audit](../../PLAN_CAPABILITIES_CARPLAY.md)). Row 100 ExtensionKit/Foundation remains unsupported: the status-like inventory is Swift-only and asynchronous, while host/browser UI needs extension process/XPC lifecycle; B221 confirms no ExtensionFoundation Objective-C/C API or standalone host/process support query ([D86/B221 audit](../../PLAN_CAPABILITIES_EXTENSIONKIT.md)). Row 102 ContactProvider remains unsupported because `ContactProviderManager.isEnabled` is Swift-only and reports person-enabled state rather than extension health or sync; B224 reconfirms no C/Objective-C route, and its throwing initializer may register a default domain ([D87/B224 audit](../../PLAN_CAPABILITIES_CONTACTPROVIDER.md)). Row 099 FileProvider has B75 partial support for registered-domain presence and B82 for its returned count in the caller app; its scoped compile/Clippy/rustdoc/link gates passed, but this does not prove provider enablement, sync, or file access ([D85/B75](../../PLAN_CAPABILITIES_FILEPROVIDER.md)). Row 101 BrowserEngineKit has generated bindings, but no host/process/XPC implementation or Apple entitlement approval; B218 confirms `BEProcessCapabilityGrant.isValid` requires an already-granted live process handle and does not establish app entitlement or engine support ([D88/B218 audit](../../PLAN_CAPABILITIES_BROWSERENGINEKIT.md)). Row 103 ManagedApp/Distribution has Swift-only configuration and catalog APIs; B227 confirms these are admin/entitlement-scoped, not generic managed-device status ([D89/B227 audit](../../PLAN_CAPABILITIES_MANAGEDAPP.md)). Row 104 MarketplaceKit remains unsupported: B230 confirms its source/region values are async Swift-only and its Objective-C action button is a user install control, not a status query ([D90/B230 audit](../../PLAN_CAPABILITIES_MARKETPLACEKIT.md)).
Row 105 MatterSupport is partial (`B`) for B233’s iOS 17+ `MatterAddDeviceRequest.isSupported` query only. It reports support for that request API, not generic Matter, Thread, accessory, entitlement, ecosystem, commissioning, or setup readiness; it creates no request and presents no UI ([D91/B233 plan](../../PLAN_CAPABILITIES_MATTERSUPPORT.md)).
Row 089 AdAttributionKit/AdServices is partial (`B`) for B239’s iOS 18+ `AppImpression.isSupported` query only. It reports support for app impressions on this device; it does not cover `Postback.isSupported`, ad-network registration, campaign eligibility, consent, attribution delivery, or general framework readiness. AdServices token generation remains network-dependent and is not a status query ([D74/B185 audit and B239 plan](../../PLAN_CAPABILITIES_AD_ATTRIBUTION.md)).
The four D1 portable contracts cover application lifecycle, sandbox files/directories, preferences,
and foreground HTTP values. D7 adds the read-only `framework-resources` contract for exact paths
inside packaged resources. B10 adds an iOS main-bundle backend for exact ordinary files, with an iOS
4.0 API floor; no live read, localization, asset-catalog access, or symlink-containment claim is made.
D8 adds borrowed, syntax-validated RFC 3986 `Uri` and `UriReference` values in `framework-format`
([guide](uri.md)); they do not normalize, percent-decode, or resolve references. B11 separately uses
`Uri` to request an external HTTPS URL handler; it does not guarantee Safari or page load.
B64 separately constructs an HTTPS-only `SFSafariViewController` and exposes a borrowed
`UIViewController` for host-managed presentation and dismissal ([iOS guide](../ios/safari.md)). The
host owns the UIKit lifecycle; construction does not guarantee presentation, a URL request, page
load, or visible content. The SafariServices declaration floor is iOS 9.0, while the scoped Rust
device link floor is iOS 10.0 and the Simulator gate uses iOS 14.0. B65 adds only single-precision
vDSP vector addition on iOS; no portable math contract, parity, or performance claim is made
([iOS guide](../ios/accelerate.md)).
D5 adds a partial portable plain-text clipboard contract; B6 adds an iOS general-pasteboard backend
with documented privacy behavior and item replacement, and B84 adds a presence-only `hasStrings`
preflight that does not load content. No live privacy prompt or paste behavior is claimed. B1 adds iOS sandbox file and
`NSUserDefaults` backends for three rows. B17 adds a separate `IosFileCoordinator` extension for
synchronous Foundation coordination of caller-supplied file URLs ([guide](../ios/file-coordination.md));
it does not add provider, picker, or security-scope lifecycle support or change D1 sandbox paths.
B2 adds a public Keychain generic-password backend for opaque bytes with explicit protection
requirements; no live Keychain test is claimed. B3 adds the Foundation `URLSession` foreground
HTTP backend and converts `NSString` response headers as UTF-8 plus `NSNumber` values through
Foundation `stringValue()`; numeric values do not preserve original wire spelling. No runtime request
or Apple parity test is claimed. D10 adds durable GET file-download
values; B13 adds a Foundation background `URLSession` backend using B14 file adoption ([guide](../ios/transfer.md)). It requires app-owned event forwarding and does not claim runtime relaunch or force-quit evidence; an ambiguous commit crash can leave the last durable status `Active`. D3 adds a partial portable local-
notification contract; B4 adds an iOS local-notification backend but not remote push. D4 adds a
partial portable one-shot location contract; B5 adds an iOS Core Location backend for foreground
current location only, with no continuous updates, geofencing, significant-change monitoring, or
background operation. D6 adds a partial portable outgoing-share contract for UTF-8 text and URL
text; B7 adds UIKit `UIActivityViewController` presentation for those items from an explicit
same-window presenter/source-view context, with an iOS 8.0 floor. No live share UI, recipient
delivery, or unforeseen UIKit presentation recovery is claimed. B8 adds synchronous accessibility
metadata setters for a borrowed `UIView`; its iOS 6.0 API floor is declaration-derived, while this
host's SDK/link deployment minimums are iOS 12.0 for device and iOS 14.0 for simulator. No live
VoiceOver behavior is claimed. B9 adds a synchronous one-action `UIAlertController` acknowledgement
alert with an iOS 9.0 API floor ([iOS guide](../ios/presentation.md)). UIKit's presentation method
has no failure callback; no live display or dismissal is claimed. The UIKit slice remains partial: a
Rust-owned app delegate and one window, plus D14's reusable container, label, button, finite Frame,
owned target/action callback, and capability-specific native handles.
The manifest counts thirty-six portable contracts as implemented and twenty-two as partial (58 portable-contract rows total). D9 adds owned
notification-response values in `framework-notifications` ([guide](../notification-responses.md));
B12 adds an opt-in iOS local-response delegate bridge ([guide](../ios/notification-responses.md)).
The app must retain its handle and serialize access to the shared delegate; the callback queue is
unspecified, invalid and remote-push responses are dropped, and no live-delivery claim is made.
D11 adds `framework-motion` ([guide](motion.md)) with a one-shot raw accelerometer contract; B15
adds the iOS backend ([guide](../ios/motion.md)). Device/simulator compile and lint checks do not
claim physical-sensor behavior. D12 adds `framework-auth` ([local-authentication guide](authentication.md))
for one-shot biometric-only or device-owner policy checks; B16 adds the iOS LocalAuthentication
backend ([iOS guide](../ios/authentication.md)). DeviceOwner requires iOS 9.0 and BiometricsOnly
starts at iOS 8.0. No live prompt, biometric-data, or identity-proof claim is made. D13 adds
`framework-data` ([data guide](data.md)) for exact borrowed byte/UTF-8 views, explicit owned copies,
and allocation-transfer conversions; B19 adds explicit raw-byte copies to and from immutable Core
Foundation `CFData` through the [iOS data guide](../ios/data.md), while native UTF-8/string
conversion remains unsupported. D15 adds the
[portable connectivity contract](connectivity.md) for one informational path snapshot; B18 adds
the separate [iOS Network.framework backend](../ios/connectivity.md). It does not preflight or gate
requests, and compile/link checks do not prove live path or cancellation behavior. B20 adds a strict
`NSURL` adapter for D8's absolute `Uri` values ([iOS URL guide](../ios/url.md)); it rejects input
Foundation does not accept without automatic invalid-character encoding, keeps the exact source
text, and has an iOS 17.0 API floor. Its compile/link checks do not prove parser acceptance or
component parity.

D16 adds exact positive-area intersection for finite `Frame` values in the portable UI guide.
B21 adds one CoreGraphics-backed `CGRectIntersection` operation through the [iOS UI guide](../ios/ui.md).
The portable and iOS implementations share edge/touching semantics; CoreGraphics runtime parity
remains untested, so the broad graphics row is still partial. D17 adds finite rational `MediaTime`
values with exact comparison; B22 maps them to CoreMedia `CMTime` by value ([portable media guide](media.md),
[iOS media guide](../ios/media.md)). The adapter has an iOS 4.0 API floor; no media buffers, capture,
playback, or AVFoundation API is included, and the probes do not test runtime behavior.

D18 adds portable finite `FontMetrics` values and a `TextMetricsBackend` contract for system-font
ascent, descent, and leading. B23 queries those metrics through public CoreText C calls ([portable
guide](text-metrics.md), [iOS guide](../ios/text-metrics.md)); the SDK API floor is iOS 3.2. It does
not shape text or measure width, line breaks, paragraphs, custom fonts, or Dynamic Type. The
device/Simulator Release probes check signatures, layout, and imports but do not call CoreText or
establish `UILabel` parity.

D20/B25 adds one synchronous `BGAppRefreshTask` path using public BackgroundTasks APIs. Task launch
time is OS-controlled; host task registration and `BGTaskSchedulerPermittedIdentifiers` /
`UIBackgroundModes` entries remain app-owned. The compile/link/import gates do not establish task
delivery, expiry handling, relaunch, or performance. It does not implement `BGProcessingTask` or
replace B13 background URLSession downloads; D23/B28 separately covers one UIKit execution lease.

D19 adds `framework-connection` for one bounded outbound TLS-over-TCP byte stream; B24 uses public
Network.framework C APIs ([portable guide](connection.md), [iOS guide](../ios/connection.md)).
Send/receive chunks are limited to 1 MiB and operations are serialized; it does not add listeners,
UDP, Bonjour, HTTP, or WebKit. Local-network privacy metadata remains host-owned when applicable,
and compile/link/import evidence is not a live peer or TLS test.

D21 adds `framework-image` for image-zero encoded dimensions and total source count; B26 reads only
those metadata values through ImageIO ([portable guide](image-metadata.md), [iOS guide](../ios/image-metadata.md)). It does not request raster output or claim general CoreImage/ImageIO support

D22 adds `framework-photos` for explicit read/write authorization status and request; B27 uses
PhotoKit's read/write access level and preserves `Limited` separately ([portable guide](photos.md),
[iOS guide](../ios/photos.md)). It does not enumerate assets, request image data, edit assets, or
present a picker; the host app owns `NSPhotoLibraryUsageDescription`

D23/B28 adds one UIKit background-execution lease with explicit end and a cooperative expiry
signal ([portable guide](background-execution.md), [iOS guide](../ios/background-execution.md)).
It does not promise extra runtime, future launch, continued execution after expiry, or work
completion; app extensions are unsupported.

D24/B29 adds Contacts authorization status and explicit request only ([portable guide](contacts.md),
[iOS guide](../ios/contacts.md)). `Limited` remains distinct from full access; the host app owns
`NSContactsUsageDescription`. Enumeration, fetch, edits, picker UI, and live consent/data behavior
remain out of scope.

D25/B30 adds EventKit Calendar event-authorization status and explicit full-access request only
([portable guide](calendar.md), [iOS guide](../ios/calendar.md)). `WriteOnly` remains distinct from
`FullAccess`; the host app owns `NSCalendarsFullAccessUsageDescription`. The backend targets iOS
17.0 or later and does not read or write events, request reminders, or present calendar UI.

D26/B31 adds borrowed HealthKit type values and an explicit read/share authorization request only
([portable guide](health-authorization.md), [iOS guide](../ios/health-authorization.md)). The backend
checks HealthKit availability before calls and does not infer access from request completion or
expose samples. The host app owns the HealthKit entitlement and conditional usage descriptions.

D27/B32 adds a non-prompting Bluetooth authorization snapshot through the public
`CBManager.authorization` class property. The adapter targets iOS 13.1 or later and does not create
a manager or implement radio state. D29/B34 adds explicit unfiltered foreground central discovery
with copied peer UUID/RSSI values and a bounded 32-event queue. Starting a scan may prompt and
returns before readiness/results; there is no background-delivery guarantee, connection,
peripheral/advertising operation, or radio control. Discovery identity has an iOS 8.0 floor; static
authorization remains iOS 13.1. Apps linked on or after iOS 13 need
`NSBluetoothAlwaysUsageDescription`; apps whose deployment target predates iOS 13 need both that and
`NSBluetoothPeripheralUsageDescription`.

D28/B33 adds a platform-exclusive borrowed HTTPS URL and navigation contract with a typed iOS
`WKWebView` adapter. The local typed objc2 declarations fill a generated iOS binding gap; the adapter
supports attach, back/forward state and actions, reload, and stop only. It does not provide a
JavaScript bridge, arbitrary file/HTML/data loads, a subresource firewall, browser parity, or a live
page-load guarantee. See [portable web](web.md) and [iOS web](../ios/web.md) guides.

D30/B35 adds an iCloud Drive Documents identity-presence snapshot only. It maps a nullable
`NSFileManager.ubiquityIdentityToken` to present/absent and never returns or retains the token. Nil
has multiple causes; presence does not prove container access, sync, or CloudKit account status. See
[portable iCloud identity](icloud-drive-identity.md) and [iOS iCloud identity](../ios/icloud-drive-identity.md) guides.

D31/B36 queries camera and microphone authorization status through AVFoundation only. It does not
request permission, enumerate devices, create capture/audio sessions, or access media samples.
`NSCameraUsageDescription` and `NSMicrophoneUsageDescription` apply before a host requests access or
attempts capture; this query does neither. See [portable media authorization](media-authorization.md)
and [iOS media authorization](../ios/media-authorization.md) guides.

D32/B37 queries `NFCReaderSession.readingAvailable` only. It reports reader support, not permission,
session readiness, tag discovery, or tag I/O; no NFC session is created. The iOS 11.0 API floor and
conservative `NFCReaderUsageDescription` caveat are documented in the [NFC](nfc.md) and
[iOS NFC](../ios/nfc.md) guides.

D34/B39 reports only Nearby Interaction's `supportsPreciseDistanceMeasurement` device capability
through `NISession.deviceCapabilities` on iOS 16.0+. It does not report permission, peer
compatibility, session readiness, or operation success; no session or ranging operation is run. See
[Nearby Interaction](nearby-interaction.md) and [iOS Nearby Interaction](../ios/nearby-interaction.md).

D35/B40 reports only the calling app's App Tracking Transparency authorization status through
`ATTrackingManager.trackingAuthorizationStatus` on iOS 14.0+. It does not request authorization,
access IDFA, or track. `NSUserTrackingUsageDescription` remains required host configuration for ATT
API use. See [privacy authorization](privacy-authorization.md) and
[iOS tracking authorization](../ios/tracking-authorization.md).

D36/B41 reports only whether `MTLCreateSystemDefaultDevice` returns a device object on iOS 8.0+.
It drops the temporary retained object and submits no work; no MetalKit, rendering, compute, feature,
or performance support is claimed. See [Metal](metal.md) and [iOS Metal](../ios/metal.md).

D37/B42 reports only the DeviceCheck and App Attest `isSupported` values, with iOS floors of 11.0
and 14.0. It does not create keys or tokens, attest or assert, contact a service, or establish trust.
D38/B43 reports only `WCSession.isSupported()` on iOS 9.0+; it does not retrieve or activate a
session, inspect pairing, or communicate. D39/B44 reports only whether the current
`EAAccessoryManager.connectedAccessories` list is empty on iOS 3.0+; it does not identify hardware,
open a session, or communicate. D40/B45 reports only the legacy `RPScreenRecorder.isAvailable`
value on iOS 9.0+; Apple currently marks it deprecated and recommends ScreenCaptureKit, which this
slice does not implement. See the [DeviceCheck guide](device-integrity.md), [Watch Connectivity
guide](watch-connectivity.md), [ExternalAccessory guide](accessory.md), [ReplayKit guide](replaykit.md),
and their [iOS guides](../ios/device-integrity.md), [Watch Connectivity](../ios/watch-connectivity.md),
[ExternalAccessory](../ios/external-accessory.md), and [ReplayKit](../ios/replaykit.md).

D41/B46 reports only whether the built-in SoundAnalysis version 1 classifier request is recognized
on iOS 15.0+. It creates no analyzer, supplies no audio, requests no microphone access, and does not
cover ShazamKit. See [SoundAnalysis](sound-analysis.md) and [iOS SoundAnalysis](../ios/sound-analysis.md).

D42/B47 reports only whether any other app is playing audio at query time, through
`AVAudioSession.isOtherAudioPlaying` on iOS 6.0+. It includes ambient-category audio, but does not
identify the source, report this app's playback, or expose Now Playing metadata or media control.
See [Other Audio](other-audio.md) and [iOS Other Audio](../ios/other-audio.md). D43/B48 adds a portable
HDR-eligibility snapshot through `AVPlayer.eligibleForHDRPlayback` from iOS 13.4; it does not inspect
a specific asset or start playback ([portable guide](playback.md), [iOS guide](../ios/playback.md)).
D44/B49 adds platform-exclusive mail/text availability and SharedWithYou software-support status;
it does not compose or send messages, inspect accounts, or access collaboration data ([MessageUI](../ios/message-ui-support.md),
[SharedWithYou](../ios/shared-with-you-support.md)). D45/B50 asks VideoToolbox only whether the system reports hardware decode support for a caller
codec FourCC; it does not encode, decode frames, inspect assets, or reserve decoder resources
([portable guide](video-codec.md), [iOS guide](../ios/videotoolbox.md)). D46/B51 reads only PassKit's general Apple Pay capability
predicate on iOS 10.0+; it does not inspect cards, merchant networks, or process a payment
([guide](apple-pay.md), [iOS guide](../ios/apple-pay-availability.md)).

D47/B52 adds one CloudKit account-status snapshot from the app's default container; it does not read
CloudKit data, observe account changes, or prove container/database access
([portable guide](cloudkit-account-status.md), [iOS guide](../ios/cloudkit-account-status.md)).

D48/B53 reads only SafetyKit's Crash Detection device-support bit; it does not claim event
entitlement or authorization, receive crash events, or provide emergency response. The SDK header
states a class entitlement, but the getter-specific prerequisite remains unspecified
([iOS guide](../ios/safetykit.md)).

D49/B54 adds one `WorldTrackingSupport` value and queries ARKit configuration support only; it
does not create a session, access the camera, request camera permission, or claim active tracking
([portable guide](arkit.md), [iOS guide](../ios/arkit.md)).

D50/B55 adds an allocation-free portable local-player authentication-status contract and one
non-prompting Game Center Boolean read; it does not initialize authentication, expose identity or
game data, or observe changes. A configured app requires the signed Game Center entitlement
([portable guide](gamekit-status.md), [iOS guide](../ios/game-center-status.md)).

D51/B56 checks only whether the iOS Core ML compute-device list is nonempty; it does not load a
model or run inference, and it makes no claim that a particular model can run
([iOS guide](../ios/core-ml-device-status.md)).

D52/B57 checks only whether a caller-supplied text-recognition revision appears in Vision's
supported revision set. It creates no request, reads no image, and claims no recognition or model
readiness result ([portable guide](vision.md), [iOS guide](../ios/vision.md)).

D53/B58 reads only the app's saved Speech authorization status. It does not request permission,
create a recognizer, accept audio, or start recognition; authorization does not establish service
availability or recognition success ([iOS guide](../ios/speech-status.md)).

D54/B59 checks only the on-device asset state for Apple’s built-in English contextual model. It does
not load the model, accept text, compute vectors, request assets, or guarantee a later model
operation ([iOS guide](../ios/natural-language-status.md)).

D55/B60 exposes only the deprecated StoreKit 1 `SKPaymentQueue::canMakePayments` bit. It does not
create a queue, inspect products or accounts, present payment UI, or process transactions. D58/B63
adds only the StoreKit 2 `AppStore.canMakePayments` status; product, transaction, and portable
commerce contracts remain out of scope ([D55 iOS guide](../ios/storekit-status.md),
[D58 iOS guide](../ios/storekit2-status.md)).

D56/B61 exposes a portable `RoomPlanDeviceSupport` value and the iOS 16.0+
`RoomCaptureSession.isSupported` predicate through a compiler-verified `swiftcall` thunk. It does not
create a session, access camera/LiDAR frames, request permission, or claim scan success ([portable
guide](roomplan.md), [iOS guide](../ios/roomplan-status.md)).

D57/B62 adds an iOS 4.0+ default-video-device presence query through
`AVCaptureDevice.defaultDeviceWithMediaType(AVMediaTypeVideo)`. It adds no portable camera contract,
authorization query, capture session, media access, or UI ([iOS guide](../ios/camera-device-status.md)).

D58/B63 exposes the iOS 15.0+ `AppStore.canMakePayments` Boolean through a weak-import
`swiftcall` thunk. The absent-symbol fallback is `false`; the query adds no product, account,
entitlement, purchase UI, or transaction behavior ([iOS guide](../ios/storekit2-status.md)).

D59/B65 exposes only iOS 4.0+ single-precision `vDSP_vadd` vector addition through equal-length
borrowed `f32` slices. It adds no portable math contract, other Accelerate operation, or performance
claim ([iOS guide](../ios/accelerate.md)).

B66 adds only the iOS 2.0+ CommonCrypto `CC_SHA256` operation for owned digest bytes. The portable
crypto facade and system/hardware key escape remain unimplemented; no replacement, parity, or
performance claim is made.

B67 adds only the iOS ModelIO `MDLAsset.canImportFileExtension` query. It reports extension-level
support only; it does not load or parse a file, render an asset, or claim GPU support ([iOS guide](../ios/modelio-status.md)).

B68 adds only whether `MPSGetPreferredDevice` returns a device with default options. It does not
submit GPU work or establish support for any MPS operation or workload ([iOS guide](../ios/mps-status.md)).

B69 adds one borrowed P-256 public-key import and ECDSA/SHA-256 message-verification suitability
query. It does not verify a signature, use or persist a private key, or access the Secure Enclave
([portable guide](key-support.md), [iOS guide](../ios/key-support.md)).

B70 adds a portable finite parent-local position contract and a detached iOS `SKNode` create/get/set
facade for `position` only. The API floor is iOS 7.0; the device/Simulator link probes use 12.0/14.0
SDK validation minimums. It does not add SceneKit, scenes, rendering, hierarchy, animation, or physics
([portable guide](spritekit-node-position.md), [iOS guide](../ios/spritekit-node-position.md)).

B71 reads only the point-in-time MediaPlayer library authorization status on iOS 9.3+; it does not
request access, read library items, or contact Apple Music services ([iOS guide](../ios/media-library-status.md)).

B72 reads `CXCallObserver.calls` once and copies a count plus aggregate outgoing, connected, on-hold,
and ended flags. The synchronous initial read may block; this does not expose call objects, UUIDs,
caller data, callbacks, call control, providers, PushKit, or audio ([iOS guide](../ios/call-observer.md)).

B73 adds finite caller-supplied map geometry: coordinate/map-point conversion and distance. It does
not add map UI, user location, permissions, network service, search, or directions
([portable guide](mapkit.md), [iOS guide](../ios/mapkit.md)).

B74 reads only `NSUserActivity.isClassKitDeepLink` from a caller-owned activity on iOS 11.3+. It
does not access `CLSDataStore`, assignment content, context identifiers, or user identity. The
getter has no documented entitlement; Schoolwork data sharing is a separate host capability
([ClassKit plan](../../PLAN_CAPABILITIES_CLASSKIT.md)).

| Family | Rows | iOS class count |
| --- | ---: | --- |
| Core app/UI | 9 | `B`: 9 (2 partial UIKit example rows; 2 partial reusable UI-control rows; 1 partial native-escape row; 1 partial clipboard row; 1 partial share row; 1 partial accessibility row; 1 partial acknowledgement-alert row); `X`: 0 |
| Files/data/preferences | 7 | `B`: 7 partial; `X`: 0 |
| Security/auth | 7 | `B`: 7 partial; `X`: 0 |
| Networking/web | 7 | `B`: 7 partial, including external HTTPS handling, WebKit navigation, and host-presented SafariServices construction; `X`: 0 |
| Notifications/background | 6 | `B`: 5 partial; `X`: 1 |
| Sensors/connectivity | 9 | `B`: 6 partial; `X`: 3 |
| Camera/audio/media | 8 | `B`: 8 partial including VideoToolbox hardware-decode support; `X`: 0 |
| Graphics/GPU | 9 | `B`: 9 partial finite-Frame geometry, CoreGraphics frame-intersection, CoreText system-font-metrics, ImageIO metadata, Metal device-presence, Accelerate vDSP vector-add, ModelIO extension-support, MPS preferred-device, and SpriteKit node-position rows; `X`: 0 |
| ML/vision/language | 5 | `B`: 4 partial Core ML compute-device, Vision revision-status, Speech authorization-status, and Natural Language English-model asset rows; `X`: 1 |
| Personal data/system stores | 8 | `B`: 6 partial Photos, Contacts, Calendar, HealthKit authorization, SafetyKit availability, and Family Controls raw-status rows; `X`: 2 |
| Cloud/accounts/communication | 10 | `B`: 8 partial including CloudKit, CallKit, ClassKit marker, Game Center, and MessageUI/SharedWithYou status; `X`: 2 |
| Commerce/services | 7 | `B`: 4 partial Apple Pay, legacy StoreKit, StoreKit 2 purchase-ability, and MediaPlayer library-authorization status; `X`: 3 |
| Maps/AR/spatial | 5 | `B`: 3 partial MapKit geometry, ARKit world-tracking, and RoomPlan device-support rows; `X`: 2 |
| Extension/entitlement capabilities | 13 | `B`: 4 partial; `X`: 9 |
| Compiler/build-host capabilities | 3 | `X`: 3 |

For unverified platform metadata the manifest uses `null`, not an inferred empty requirement. A
verified empty list means the row's source documents no required item for that field. The B UI
facts come from [`docs/ios/runtime.md`](../ios/runtime.md) and the `ios-minimal` example; no iOS
minimum version is stated there, so the manifest leaves it unknown. No entitlement, permission, or
framework requirement is inferred for an `X` row. A `true` native-escape value names the handle
documented for that row: UIKit example handles or a capability-specific borrowed native object.
The C7 [App Intents audit](../swift-abi/APP_INTENTS_STAGE0.md) found no stable Rust/C metadata input
or processor API on Xcode 26.6; the Stage 1 runtime path remains unsupported, with no fake capability
API. Re-audit on the Xcode 27.x baseline.

See [the D1 API guide](app-data.md) for the four portable crate contracts, [the secure-storage
guide](secure-storage.md) for D2's opaque-byte contract, [the data guide](data.md) for D13's
borrow/copy/ownership-transfer behavior, [the connectivity guide](connectivity.md) for D15's
advisory path status, [the UI guide](ui.md) for D14's finite frame and native label/button
slice, and [the font-metrics guide](text-metrics.md) for D18's limited portable values. These guides detail ownership, copy, atomicity, async/cancellation,
errors, and runtime limits. D3–D81 are tracked in separate named subplans; D15's implemented
contract and backend are counted as partial support in row 028, while B21's finite CoreGraphics
operation is counted in row 055, B22's finite time value is a partial slice of row 049, B23's
system-font metrics are a partial slice of row 056, B26 is a partial slice of row 057, B27 is a
partial slice of row 068, B28 is a partial slice of row 035, B29 is a partial slice of row 069,
and B30 is a partial slice of row 070, B31 is a partial slice of row 071, B32 and B34 are partial
slices of row 039, B33 is a partial slice of row 029, B35 is a partial slice of row 077, B36 is
a partial slice of rows 046 and 047, B37 is a partial slice of row 040, B39 is a partial slice
of row 041, B40 is a partial slice of row 023, B41 is a partial slice of row 058, B42 of row 022,
B43 of row 078, B44 of row 043, B45 of row 052, B46 of row 053, B47 of row 051, B48 of row 048, B49 of rows 081–082, B50 of row 050, B51 of row 088, B52 of row 076, B53 of row 073, B54 of row 094, B55 of row 079, B56 of row 063, B57 of row 064, B58 of row 066, B59 of row 065, B60 of row 087, B61 of row 096, B62 of row 046, B63 of row 086, B64 of row 030, B65 of row 060, B66 of row 018, B67 of row 061, B68 of row 059, B69 of row 019, B71 of row 090, B72 of row 083, B73 of row 093, and B74 of row 080. Workstream D remains incomplete; these bounded slices do not claim family-wide parity.
The D3 [local-notification guide](../notifications.md) describes the portable scheduling contract;
the B4 [iOS guide](../ios/notifications.md) describes the local-only native backend and its runtime
limits. The D4 [location guide](location.md) describes the one-shot portable current-location
contract; the B5 [iOS guide](../ios/location.md) documents its Core Location backend and limits.
The D5 [sharing guide](sharing.md) describes the plain-text clipboard contract; the B6 and B84
[iOS guide](../ios/sharing.md) documents the general-pasteboard backend, presence-only preflight,
and native privacy limits.
The D6 [share guide](share.md) describes outgoing text and URL-text values; the B7 [iOS
guide](../ios/sharing.md) documents UIKit presentation context, lifecycle, result, and evidence
limits.
The B8 [accessibility guide](../ios/accessibility.md) documents the borrowed-view setters, trait
replacement semantics, API floor, and absence of live VoiceOver evidence.
The D14 [portable UI guide](ui.md) documents finite local-coordinate frames and owned button
callbacks; the [iOS UI guide](../ios/ui.md) documents UIKit control ownership, main-thread rules,
borrowed native handles, and unverified live behavior. Four partial matrix rows record its bounded
view/control, target/action, native-handle, and finite-Frame surfaces; a general geometry library or
cross-capability escape facade is not provided. Window, controller, scene, navigation, general
layout, and accessibility systems remain outside its scope.

The optional foreign-language Keychain surface is documented in the
[secure-storage C ABI guide](../bindings/secure-storage.md); it does not change the Rust-native
call path or the iOS capability classification.
