# PLAN.md — V1 iOS Rust Framework Implementation

## Status

Planning set generated against repository:

- Repository: `Biddlebaddleboo/jcdigitalsolutions-ios-rust-framework`
- Target branch: `main`
- Verified planning baseline: `3fa8222e6e6fff6370ac461a00fa6e61e9e84a6d`
- Required build baseline: macOS with Xcode 27.x; observed local host is Xcode 26.6 build 17F113, below that baseline
- Primary V1 runtime target: iOS arm64
- Secondary V1 target: iOS simulator arm64
- Future targets preserved architecturally: macOS, Android, Windows, Linux, Web/WASM, and a future compact/32-bit-pointer internal mode

This plan is authoritative for V1 implementation. At the planning baseline, the repository held architecture and research docs only. The current checkout has a Cargo workspace, portable foundation, tooling, selected capability contracts and iOS backends, bindings, and validation. Unimplemented paths remain proposed additions; see the workstream plans and capability status manifest for current scope.

In the capability manifest, `X` means no supported Rust facade and iOS backend exist in the current workstream for that row. It does not mean Apple's API is unavailable or that Rust cannot call it; some rows await a bounded contract, an implementation, or platform evidence. The row's `status_reason` records the known gap.

Current integrated matrix: 98 of 114 rows are partial (`B`) (86.0% row coverage); 16 remain `X`, all with specific scope or toolchain gaps. B228 adds the retained attributed user-input-label array, B234 adds an owned accessibility-path copy, B243 adds the raw accessibility-trait mask, B246 adds a selector-guarded direct-touch-options snapshot, and B249 adds a main-thread VoiceOver-running snapshot and B252 adds a main-thread Switch Control-running snapshot with an iOS 8+ caller-availability requirement, and B255 returns a Guided Access-gated AssistiveTouch status; B235 adds the directory object’s allocated-size snapshot; B242 adds the logical byte length across a regular file’s forks; B244 adds the data-fork allocated-byte snapshot, and B248 adds the resource-fork allocated-byte snapshot, and B251 adds the resource-fork logical byte length. B237/B238/B240/B241 decline directory metadata with no matching app-data operation; B247 declines a fork count with no matching facade operation; B253 declines a duplicate all-fork allocation scalar already covered by B109/B244/B248. See the focused workstream plans. This is row coverage, not whole-plan completion: no capability family is fully complete. B81 adds point-in-time `connectedScenes` activation-state counts to row 001, not lifecycle events or scene visibility; see `PLAN_IOS_SCENE_SNAPSHOT.md`. B84 adds a main-thread `UIPasteboard.hasStrings` presence snapshot; it does not read text or alter clipboard permission semantics, and it does not change the row count; see `PLAN_IOS_CLIPBOARD.md`. B85 creates an opaque plain-bookmark value with implicit security scope omitted and allows safe resolution only through that value; it grants no access and does not change the matrix count; see `PLAN_IOS_BOOKMARK_CREATION.md`. B86 adds a raw signed local-notification authorization-status snapshot through the existing prompt-free UserNotifications settings query; it preserves provisional, ephemeral, and unknown values without changing portable D3 or matrix counts; see `PLAN_IOS_NOTIFICATIONS.md`. B87 adds an iOS 14+ prompt-free full/reduced location-accuracy authorization snapshot, preserving unknown signed values; it does not change portable D4 or row coverage; see `PLAN_IOS_LOCATION.md`. B88 adds one prompt-free `UNNotificationSettings` snapshot of raw alert, sound, and badge values, preserving unknown signed values without changing portable D3 or matrix counts; see `PLAN_IOS_NOTIFICATION_SETTINGS.md`. B90 adds an iOS-only no-follow regular-file byte-length snapshot under a validated `AppPath`, without reading file contents or changing portable file semantics; see `PLAN_IOS_FILE_SIZE.md`. B91 adds one prompt-free notification-settings snapshot with raw Notification Center/Lock Screen values and optional critical-alert, time-sensitive, and scheduled-delivery settings; it preserves unknown values and does not change D3 or delivery claims; see `PLAN_IOS_NOTIFICATION_SETTINGS_EXTENDED.md`. B92 adds UIKit's `Adjustable` accessibility trait and does not add its two required actions; see `PLAN_IOS_ACCESSIBILITY.md`. B93 adds an iOS-only `FileKind` lookup for one validated `AppPath`, with no content read or portable contract change; see `PLAN_IOS_ENTRY_KIND.md`. B94 adds raw CarPlay and optional Siri-announcement settings from one prompt-free query; it does not expose the user's preview-privacy choice or imply connection/presentation readiness; see `PLAN_IOS_NOTIFICATION_SETTINGS_SURFACES.md`. B95 adds UIKit's `NotEnabled` metadata trait only; it does not disable the view or block interaction; see `PLAN_IOS_ACCESSIBILITY.md`. B96 adds a no-follow POSIX data-modification timestamp snapshot for one `AppPath`; the value is not a content version or reliable change token; see `PLAN_IOS_FILE_MODIFICATION_TIME.md`. B97 adds only an iOS 16.1+ count of unmanaged Background Assets queue entries; it does not prove support, host configuration, or asset installation; see `PLAN_CAPABILITIES_BACKGROUND_ASSETS.md`. B98 adds UIKit's `KeyboardKey` accessibility metadata trait only; it adds no keyboard or key-event handling; see `PLAN_IOS_ACCESSIBILITY.md`. B99 adds an iOS-only no-follow `(st_dev, st_ino)` snapshot for near-time comparison; it is not a persistent ID or retained handle; see `PLAN_IOS_FILE_IDENTITY.md`. B100 adds UIKit's `UpdatesFrequently` accessibility trait for a label/value that changes often; the caller owns updates and policy, and the adapter promises no poll or cadence; see `PLAN_IOS_ACCESSIBILITY.md`. B101 does not add entry creation time: Darwin may place `ctime` in `st_birthtime` when birth time is unavailable, with no per-entry signal to distinguish it; see `PLAN_IOS_FILE_CREATION_TIME.md`. B102 adds UIKit's `PlaysSound` accessibility metadata trait; the host must supply activation sound, and the adapter adds no audio behavior; see `PLAN_IOS_ACCESSIBILITY.md`. B103 adds raw iOS POSIX permission/special bits for one sandbox entry; it does not report effective access; see `PLAN_IOS_FILE_PERMISSION_BITS.md`. B104 adds UIKit's `CausesPageTurn` accessibility trait; the host must implement `accessibilityScroll:` and update represented page content. B105 adds a regular-file hard-link count only, not alias paths or exclusive ownership. B106 adds UIKit's `StartsMediaSession` accessibility trait only when activation starts a media session. B107 adds a no-follow POSIX status-change timestamp snapshot; it is not a content version or reliable change token. B108 adds UIKit's `AllowsDirectInteraction` trait for caller-marked content that supports direct touch; the adapter routes no input or `DirectTouchOptions`. B109 adds `IosFiles::regular_file_allocated_blocks_512` as a no-follow `st_blocks` snapshot in 512-byte units for regular files; it does not claim exact physical use or exclusive allocation. B110 adds an iOS 26.0+ count of asset-pack entries parsed from caller-supplied unmanaged Background Assets JSON; it does not inspect host setup or local asset status. B113 adds owned, deterministically sorted manifest descriptors containing identifier, download size, and version only; B116 adds optional Rust-owned userInfo JSON bytes from caller-supplied input. Neither reports local or server state. B119 found no further safe read-only manifest field in the installed SDK/binding; language is beta for iOS 27. B124 counts only returned unmanaged queue entries whose `BADownload.isEssential` is true at iOS 16.4+; it is not an installed-asset or all-device count. B127 adds an iOS 16.1+ asynchronous count of returned queue entries whose `priority` differs from `BADownloaderPriorityDefault`; it does not read state or identifiers. B111 adds UIKit's `SummaryElement` trait for a caller-supplied summary of app conditions, settings, or state; the adapter makes no presentation-timing claim. B112 adds a no-follow POSIX `st_atime` snapshot under `AppPath`; it is neither a guaranteed access log nor a reliable change token. B114 adds a synchronous read of `UIView.isAccessibilityElement`, reporting property state only, not visibility, focus, or assistive-app reachability. B117 adds nullable-property presence checks for accessibility label, hint, and value; they return no text and do not prove caller assignment. B120 checks membership for a known accessibility trait without raw-mask exposure. B123 gets and sets whether child accessibility elements are hidden, not the receiver's own element flag, visual state, or interaction state; it returns `AccessibilityApiUnavailable` when the guarded selector is absent and does not raise the iOS 4.0 package floor. B126 adds selector-guarded group-children state for logical parent groups; B129 adds selector-guarded modal-child state for actually modal content, without an assistive traversal or app-presentation claim. B132 adds selector-guarded BCP 47 accessibility-language metadata; it does not validate/normalize input, read the effective assistive-app language, or invoke the iOS 17 callback. B135 adds selector-guarded `accessibilityRespondsToUserInteraction` metadata; its default can derive from other properties, and it creates no interaction handler. B138 adds selector-guarded navigation style with unknown native values represented as `None`; the SDK says this property currently affects Switch Control, not VoiceOver. B141 adds selector-guarded iOS 13.0 textual-context metadata using seven named UIKit string constants; it does not invoke the iOS 17 callback or claim assistive output. B144 adds selector-guarded `AccessibilityContainerType::{Unspecified, List, Landmark}` at iOS 11.0; unknown values map to `None`, while `DataTable` remains excluded because it needs the data-table protocol; B379 adds iOS 13 `SemanticGroup` with a caller availability guard. B147 declines `accessibilityDirectTouchOptions`: its VoiceOver touch/audio behavior needs a dedicated direct-touch contract. B115 adds raw no-follow BSD `st_flags` with named `UF_*`/`SF_*` masks while preserving unknown bits; it does not predict effective access. B118 adds no-follow raw numeric `st_uid`/`st_gid` snapshots without account identity or access claims. B121 declines `st_blksize` because it is only an I/O size hint and the file facade has no buffer or policy hook to use it. B122 declines `st_rdev` because it applies only to special entries that this file facade cannot read or write; B125 declines `st_gen` because Apple documents it as superuser-only. B128 adds `IosFiles::directory_entry_count`, a direct-entry count across all names and entry kinds without a Rust-owned name list or per-entry kind lookup; it is O(n), libc may allocate, and concurrent changes can affect the result. B131 adds `IosFiles::directory_is_empty`, which stops at the first direct child but is not a stable snapshot or deletion guard. B134 adds `IosFiles::directory_entry_kind_counts` for regular-file, directory, and `Other` totals using no-follow per-name classification; it includes byte names and errors rather than return partial counts if a name vanishes. B137 adds `IosFiles::volume_available_capacity_bytes(AppDirectory)` from checked `f_bavail * f_bsize` on the retained root descriptor. The result is volume-wide, not an app quota, reservation, or write guarantee, and host use requires an appropriate Disk Space reason in `PrivacyInfo.xcprivacy`. B146 adds `IosFiles::volume_total_capacity_bytes(AppDirectory)` using checked `f_blocks * f_bsize` from the retained root descriptor; this is mounted-volume capacity, not physical-device capacity or an app quota, and shares B137’s Disk Space privacy-manifest requirement. B140 declines `NSURLDirectoryEntryCountKey` as a cheap-count API: it is optional, filesystem-dependent, and `ATTR_DIR_ENTRYCOUNT` is only 32-bit with no guarantee of low cost. B143 declines `f_bfree` as a separate app-facing value because it includes reserved filesystem blocks that the sandbox cannot use; B137 already reports `f_bavail` for non-superusers. B149 declines `f_ffree` as app-facing metadata because it is a volume-wide free-node count, not an app/container budget or a reliable prediction that a sandbox create will succeed. HealthKit exposes only availability and explicit authorization-request flow; Bluetooth exposes authorization status and bounded foreground central discovery; camera exposes authorization status and default-video-device presence only, while microphone exposes authorization status only; NFC exposes reader support only; Nearby Interaction exposes one device-capability query; App Tracking Transparency exposes calling-app authorization status only; Metal exposes system-default device-object presence only; MPS exposes preferred-device presence only; Accelerate exposes single-precision vector addition through vDSP only; ModelIO exposes the extension-level `MDLAsset.canImportFileExtension` query only; Security/auth exposes CommonCrypto SHA-256, P-256 public-key algorithm suitability, and prior-user Sign in with Apple credential state only; DeviceCheck/App Attest expose API support only; WatchConnectivity exposes session-object support only; ExternalAccessory exposes app-visible connected-list presence only; ReplayKit exposes legacy recorder availability only; SoundAnalysis exposes built-in classifier recognition only; Core ML exposes compute-device-list non-emptiness only; Vision exposes one text-recognition revision-support query only; Speech exposes a saved authorization-status snapshot only; NaturalLanguage exposes one English contextual-model asset-status query only; other-audio status exposes only whether any other app is playing audio; HDR playback exposes only system eligibility; MessageUI and SharedWithYou expose status bits only; Apple Pay exposes general device capability only; VideoToolbox exposes hardware-decode support by codec only; CloudKit exposes a one-shot account-status snapshot only; CallKit exposes call count and aggregate state flags only; ClassKit exposes an incoming-activity deep-link marker only; MapKit exposes finite point conversion and distance only; SafetyKit exposes Crash Detection device support only; ARKit exposes one world-tracking configuration-support query only; GameKit exposes a local-player authentication snapshot only; StoreKit exposes a deprecated legacy purchase-ability bit and the StoreKit 2 `AppStore.canMakePayments` status; RoomPlan exposes only its iOS device-support predicate; ProximityReader exposes only the Tap to Pay device-model predicate; WebKit exposes one bounded HTTPS view/navigation slice; SafariServices exposes one host-presented `SFSafariViewController` constructor for HTTPS only; Apple Music exposes MediaPlayer library authorization status only; iCloud exposes iCloud Drive identity-token presence only; SpriteKit exposes finite parent-local `SKNode.position` get/set only; FileProvider exposes registered-domain presence and its returned count for the caller app's own extension. B79 implements a narrow Personal VPN profile-status snapshot after preference load, preserving all six states and native load errors under `com.apple.developer.networking.vpn.api = ["allow-vpn"]`; provider and system-wide VPN support remain unsupported. See `PLAN_IOS_VPN_STATUS.md`. B78 implements an entitlement-scoped credential-state query for one prior Sign in with Apple user ID; passkeys, general sign-in readiness, and the original non-entitled scope remain unsupported; see `PLAN_CAPABILITIES_PASSKEYS.md`. B80 exposes a raw signed FamilyControls authorization-status snapshot on the main dispatch queue through a compiler-matched C `swiftcall` bridge; it does not prove entitlement presence, approval, activity-data access, or control use. Query-only entitlement semantics remain undocumented; see `PLAN_CAPABILITIES_FAMILY_CONTROLS_STATUS.md`. B236 exposes only the default Foundation Models readiness Boolean through compiler-derived Swift ABI; sessions, inference, generation, and other model APIs remain unsupported. DeviceActivity has a callable Objective-C authorization getter whose meaning and entitlement semantics are undocumented. See `PLAN_CAPABILITIES_FOUNDATION_MODELS.md` and `PLAN_CAPABILITIES_DEVICE_ACTIVITY.md`. TipKit exposes only Swift per-tip eligibility, not a global readiness or presentation result; see `PLAN_CAPABILITIES_TIPKIT.md`. B239 exposes only AdAttributionKit AppImpression.isSupported through compiler-derived swiftcall; Postback support and AdServices token generation remain outside that slice, with token generation network-dependent; see `PLAN_CAPABILITIES_AD_ATTRIBUTION.md`. Thread has only an entitlement-gated preferred-network query, not local-radio or border-router status; see `PLAN_CAPABILITIES_THREAD.md`. B245 now reads only DockKit's iOS 17+ system-tracking setting through a compiler-derived Swift ABI bridge; accessory presence, active tracking, camera state, and tracking operations remain unsupported. see `PLAN_CAPABILITIES_DOCKKIT.md`. WeatherKit native calls are Swift-only; B250 confirms the REST availability endpoint returns coordinate/data-set availability only and needs a trusted signed token plus host attribution and typed-data choices; RealityKit view construction alone exposes no useful scene/entity operations. See `PLAN_CAPABILITIES_WEATHERKIT.md` and `PLAN_CAPABILITIES_REALITYKIT.md`. B79 implements only a read-only caller-app Personal VPN profile-status snapshot with the `allow-vpn` entitlement; provider, tunnel-control, and global VPN surfaces remain unsupported; see `PLAN_CAPABILITIES_NETWORK_EXTENSION.md`. PushToTalk has no standalone support query and needs entitlement, APNs, microphone, channel, and audio lifecycle; CarPlay has no general availability query outside an entitled scene/session. See `PLAN_CAPABILITIES_PUSHTOTALK.md` and `PLAN_CAPABILITIES_CARPLAY.md`. HomeKit's authorization status is an instance query whose first manager use can prompt, so no prompt-free status facade is claimed. None is full capability-family support.
B180 adds the iOS 10 `TabBar` accessibility trait for an ordered tab list; the caller separately sets `isAccessibilityElement` to false. B183 declines `ToggleButton` because the generated static has no per-trait availability result for iOS 12–16 deployment targets. B178 declines the ambiguous volume maximum-file-size value; B181, B184, and B187 decline volume hard-link, symbolic-link, and advisory-lock support flags because the file facade has no matching operations; B190 declines the sparse-file support flag because the facade has no sparse allocation or hole operation. B200 extends row 098 with read-only `isEnabled` and `isOnDemandEnabled` snapshots after Personal VPN preference load; false enablement may reflect another configuration, and the on-demand flag does not prove rules or connection behavior. B186 adds iOS 18 selector-guarded `AccessibilityExpandedStatus` get/set; unknown getter values return `None`, and neither method invokes the callback block or expands content. B189 declines `accessibilityElements` because the untyped child array has no safe ownership, cleanup, or container-lifecycle contract in this borrowed-view adapter. B203 confirms SensorKit remains unavailable for general use: the legacy reader is deprecated, its beta replacement is absent from the installed SDK, and the research entitlement is restricted. B206 adds an iOS 16.4+ Rust future for ThreadNetwork preferred-network availability, requiring the host entitlement and Apple distribution approval; it reports no radio, active connectivity, or border-router capability. This moves row 045 to partial support; see `PLAN_CAPABILITIES_THREAD.md`. B209 adds only a synchronous iOS 16.1+ ActivityKit current-app start-eligibility snapshot, not Activity creation or updates. Row 112 became partial at B209 with 89/113 (78.8%) coverage and 24 `X` rows; B233, B236, B239, B245, and B254 moved coverage to 94/113 (83.2%) with 19 `X`; B236 covers only the default Foundation Models readiness Boolean, B239 only app-impression API support, B245 only DockKit system-tracking setting state, and B254 only a WidgetKit timeline-reload request without provider/render guarantees. B274, B280, and B291 moved coverage to 97/114 (85.1%); B339 moves current coverage to 98/114 (86.0%) with 16 `X`. B274/B280/B291 add only a previously selected AccessorySetupKit accessory count, AlarmKit authorization state, and RealityFoundation photogrammetry hardware support; B339 adds only the host-supplied per-accessory HomeKit identify-support bit. See `PLAN_CAPABILITIES_ACTIVITYKIT.md`, `PLAN_CAPABILITIES_MATTERSUPPORT.md`, `PLAN_CAPABILITIES_FOUNDATION_MODELS.md`, `PLAN_CAPABILITIES_ACCESSORY_SETUP.md`, `PLAN_CAPABILITIES_ALARMKIT.md`, and `PLAN_CAPABILITIES_REALITYKIT.md`.

Latest bounded slices in this checkpoint: B266 adds a descriptor-bound regular-file document-ID snapshot, with zero mapped to None and no inode/content/cross-volume identity claim; B272 adds the added-to-directory timestamp, B275 pairs `fstat` identity with `ATTR_CMN_GEN_COUNT` from the same open file descriptor, and B293 exposes only the opaque `ATTR_CMN_DATA_PROTECT_FLAGS` code without a named class mapping, and B296 adds an iOS-only detailed `fstatat(..., AT_SYMLINK_NOFOLLOW)` entry-kind snapshot without changing portable `FileKind`; B301 adds a filesystem-stored backup-time marker read that is not backup-completion status; B258/B261/B268/B276 and B279/B281/B284/B287/B290/B292/B295 add main-thread accessibility-setting snapshots; B297 adds selector-guarded UIView color-inversion metadata get/set, and B299 adds selector-guarded large-content-viewer preference get/set without attaching or presenting an interaction, including the combined Reduce Motion + Prefer Cross-Fade Transitions meaning, with documented caller-availability guards and no setting observers; B305 adds only a main-thread iOS 8+ Increase Contrast setting snapshot, with no color mutation or app-drawing contrast claim; B309 adds only UIKit’s Color Filters/Grayscale preference-state snapshot, not rendered colors or active-filter identity; B313 adds only a main-thread iOS 13+ Auto-Play Video Previews setting snapshot, not app-video playback behavior; B317 adds only Made for iPhone hearing-aid ear-side pairing status, with unknown bits preserved and no identity/audio-route claim; B325 adds a view-to-screen frame-coordinate conversion for `accessibilityFrame`, with no view mutation or presentation claim; B330 adds a view-to-screen path-coordinate conversion returning a new `Retained<UIBezierPath>`; B274 adds only the activated-session count of previously selected accessories; B280 adds only AlarmKit authorization state through compiler-derived resilient-enum witnesses; B291 adds only `PhotogrammetrySession.isSupported`; B298 adds the two `PhotogrammetrySession.limits` values without a session or image operation; B339 adds only per-accessory `HMAccessory.supportsIdentify` for a host-supplied live object; B359 adds descriptor-bound `ATTR_CMN_CRTIME` after a same-descriptor volume support-bit check, with mutable stored-time semantics and no `st_birthtime` fallback; B365 adds descriptor-bound `ATTR_VOL_SPACEUSED` as a volume-wide used-byte snapshot; B379 adds typed `SemanticGroup` container metadata with a caller iOS 13+ guard; B382 adds a typed main-thread `UIImageView`/`UIButton` image-size adjustment accessor with an iOS 11+ caller guard. B285 PushKit, B286 BrowserEngineKit, B294 persistent object ID, B303 SensorKit per-sensor availability, B304 `ATTR_CMN_PARENTID`, B306 `DeviceActivityCenter.activities`, B307 `ATTR_CMN_OBJTAG`, B310 `ATTR_CMN_UUID`, B315 MarketplaceKit age rating, B318 `AXShowBordersEnabled`, B319 `ATTR_CMN_GRPUUID`, B321 `ATTR_CMN_EXTENDED_SECURITY`, B322 HomeKit reachability, B323 `ATTR_CMN_FULLPATH`, B324 `ATTR_CMN_FNDRINFO`, B328 `ATTR_CMN_OBJID`, B329 `ATTR_CMN_PAROBJID`, B331 `UIAccessibilityRegisterGestureConflictWithZoom`, B332 `ATTR_CMN_FILEID`, B333 `ATTR_CMN_DEVID`, B334 `ATTR_CMN_ERROR`, B335 `UIAccessibilityZoomFocusChanged`, B336 `ATTR_CMN_RETURNED_ATTRS`, B337 `ATTR_CMN_NAME`, B338 `ManagedSettingsStore.isActive`, B340 drag/drop accessibility descriptors, B342 `ATTR_CMN_OBJTYPE`, B343 `UIAccessibilityReadingContent`, B345 `ATTR_CMN_FSID`, B346 `UIAccessibilityPostNotification`, B349 `UIAccessibilityCustomAction`, B351 `ATTR_CMN_ACCESSMASK`, B352 `UIAccessibilityCustomRotor`, B355 `accessibilityHeaderElements`, B358 data-table accessibility container API, and B361 `accessibilityElements` container ownership, B364 automation-only `automationElements`, B367 text-input forwarding, B370 `accessibilityHitTest:withEvent:`, B362 `ATTR_VOL_UUID`, B368 `ATTR_VOL_MINALLOCATION`, B371 `ATTR_VOL_ALLOCATIONCLUMP`, B373 `HMAccessory.isBlocked` thread-safety, B374 `ATTR_VOL_MAXOBJCOUNT`, B376 `DeviceActivityCenter.schedule(for:)`, B383 `DeviceActivityCenter.events(for:)`, and B392/B395/B398 volume-wide object-count/mountpoint/name attributes are documented no-go or deferred audits. B432 declines `UIGuidedAccessConfigureAccessibilityFeatures`: it mutates system accessibility settings, is documented for apps locked into Guided Access via a Single App Mode profile, and uses an asynchronous completion callback; see `PLAN_IOS_ACCESSIBILITY.md`. B274/B280/B291 first moved the integrated matrix to 97/114 (`B`, 85.1%); B339 later moved it to 98/114 (`B`, 86.0%) with 16 `X`. Other listed additions leave row count unchanged. This remains row coverage, not whole-plan completion.

Recent bounded follow-ups extend existing rows without changing coverage: B153 adds selector-guarded iOS 5 accessibility activation-point get/set over `CGPoint` in screen coordinates; callers must recompute after layout or screen-position changes, and no activation outcome is promised. B156 adds iOS 11 attributed-label set and non-empty-presence operations; UIKit copies values, couples the property to `accessibilityLabel`, and the adapter invokes no dynamic callback or speech behavior. B159 adds selector-guarded iOS 11 attributed-hint set/presence; UIKit copies values and couples this property to `accessibilityHint`, without callback or speech guarantees. B162 adds selector-guarded iOS 11 attributed-value set/presence; UIKit copies values and couples this property to `accessibilityValue`, without callback or speech guarantees. B169 adds selector-guarded iOS 13 attributed user-input-label set/presence, mirrors the plain property and preserves primary-first order, with no recognition guarantee. B171 adds selector-guarded iOS 7 `accessibilityPath` set/presence using screen-coordinate `UIBezierPath`; the caller owns geometry updates, and no highlight or activation behavior is promised. B174 adds selector-guarded `accessibilityFrame` get/set over screen-coordinate `CGRect`, with no coordinate conversion, layout, visibility, or assistive-output guarantee; it does not claim `UIAccessibilityElement` container-space behavior for `UIView`. B177 adds selector-guarded iOS 5 `accessibilityIdentifier` get/set for UI Automation, not a user-facing label; it does not validate uniqueness or invoke the iOS 17 block. `accessibilityFrameInContainerSpace` remains unsupported because this `UIView` adapter lacks the `UIAccessibilityElement` API. B173 confirms HomeKit has no prompt-free authorization query because the typed status is an instance property and first manager use can prompt. B176 reconfirms DeviceActivity Boolean getters lack documented meaning, prompt behavior, thread guarantees, and query-only entitlement semantics. B179 reaffirms PushToTalk has no standalone support/auth query; its only read-only active-channel value requires manager, delegate, restoration, and APNs lifecycle. B182 confirms CarPlay `supportsVideoPlayback` applies only to a connected session and offers no pre-session/device-support query; row 085 stays `X`. B185 confirms AdAttributionKit support scalars are Swift-only and AdServices token generation is network-dependent, not a status query; row 089 stays `X`. B191 confirms TipKit state is specific to a Swift `Tip` and has no C/Objective-C surface; B194 confirms RealityKit scene/entity APIs remain Swift-facing and `ARView` is only a shell. B197 confirms DockKit settings are Swift-only and accessory changes use a Swift `AsyncSequence`, with no integrated Rust callback boundary.  B150 adds guarded iOS 13 accessibility user-input-label metadata without a speech-match guarantee; B130/B133 copy general and essential download allowances only from the system extension callback object, and B136 classifies typed callback reasons while preserving unknown values; B152 reports only the retained volume’s `MNT_RDONLY` mount bit, and B155 exposes `f_iosize` only as an informational sizing hint. B164 queries `_PC_NAME_MAX` for direct child components of one retained app-directory root; it is not a total or nested path limit and does not guarantee a later operation. B170 adds `IosFiles::entry_effective_access` using no-follow `getattrlistat(ATTR_CMN_USERACCESS)` to report the current effective UID’s read/write/execute-search mask; it does not guarantee a later operation or bypass sandbox/file-protection checks. B172 exposes cached Foundation `RENAME_EXCL`/`RENAME_SWAP` volume-support values from `IosFiles::new`, preserving unknown values as `None`; these do not guarantee a later rename. B175 exposes cached Foundation case-sensitive-name and case-preserved-name support values as `Option<bool>` without changing `AppPath` comparison semantics. B165 declines `_PC_PATH_MAX` because the adapter walks validated components with descriptor-relative `openat`, so a whole-path limit is not its constraint. B168 declines `F_GETPROTECTIONCLASS` because the checked SDK/bindings provide no public numeric class mapping, while Foundation string values require path/URL re-resolution outside the retained-descriptor contract.  B158 declines `f_fsid`, `f_owner`, `f_type`, `f_fssubtype`, and `f_fstypename` as app-facing volume identity because they do not define a persistent/container identity or stable iOS filesystem mapping. B161 declines `MNT_NOEXEC`, `MNT_NOSUID`, `MNT_NODEV`, `MNT_QUOTA`, and `MNT_DONTBROWSE` as effective path/app authorization signals; B152 exposes only the read-only mount bit. These do not prove effective write access, download scheduling, asset installation, performance, runtime parity, or whole-plan completion. Latest bounded slices add B210/B213/B216 selector-guarded plain label, hint, and value getters with Rust-owned text and nil/empty preservation; B219/B222 add owned retained attributed label/hint values without callback or speech claims. B211 adds opaque ATTR_CMNEXT_CLONEID for pure-clone ID comparison only. B214 adds the point-in-time ATTR_CMNEXT_CLONE_REFCNT full-clone count, not partial peers, IDs, or paths; `fgetattrlist` requires an applicable approved File Timestamp reason in the host PrivacyInfo.xcprivacy. B223 adds `ATTR_CMNEXT_PRIVATESIZE` bytes not trapped in a clone/snapshot; deletion frees this space, so it is not an allocation or future-space guarantee. B225 adds the selector-guarded retained attributed-value getter, preserving nil; no callback or speech/output claim. B226 adds ATTR_CMNEXT_LINKID as an opaque ID for current-mounted-volume comparison only; no cross-mount, content, inode, clone-group, or authorization claim, and host use needs an approved File Timestamp reason. B231 declines ATTR_CMNEXT_RELPATH because it is a mount-relative physical path with hard-link ambiguity, not an AppPath. B232 declines ATTR_CMNEXT_ATTRIBUTION_TAG because no public bundle-ID mapping, app-data contract, or setter exists. B220 declines recursive directory generation count because apps cannot mark APFS directories for dir stats. Audits B218/B221/B224/B227/B230 retain capability rows 101/100/102/103/104 at X.

B78 implements a narrow, entitlement-scoped Sign in with Apple credential-state query for one prior user ID; row 021 is now partial (`B`), not passkey or general sign-in support. See `PLAN_CAPABILITIES_SIGN_IN_WITH_APPLE_STATUS.md`. B77 reads one extension point identifier from an explicit caller-supplied `.appex` path; row 113 is now partial (`B`), not build-host plist or App Intents support. See `PLAN_IOS_EXTENSION_SUPPORT.md`. D85/B75 implements a narrow caller-app FileProvider registered-domain presence snapshot in `ios-file-provider`; B82 adds a `u64` count of those domains without IDs or metadata. Its host/device/Simulator checks, strict Clippy, rustdoc, and link/import gates passed, while probes were inspected but not executed. Row 099 is partial (`B`), not full FileProvider support. See `PLAN_CAPABILITIES_FILEPROVIDER.md`. D86 found the ExtensionFoundation inventory query is Swift-only and host-scoped, while ExtensionKit UI needs host and extension lifecycle; row 100 remains `X`. D87 found ContactProvider status is Swift-only, and enablement plus extension setup do not form a bounded Rust-callable query; row 102 remains `X`. D88 found BrowserEngineKit bindings exist, but useful browser processes need approved regional entitlements, extension lifecycles, and XPC; row 101 remains `X`. D89 found ManagedApp/ManagedAppDistribution are Swift-only and provide no generic managed-device status; row 103 remains `X`. D90 found MarketplaceKit installation-source and region queries are async Swift-only and do not establish entitlement, approval, or distribution readiness; row 104 remains `X`. D91 initially found MatterSupport’s narrow API-support query was Swift-only; B233 now implements only `MatterAddDeviceRequest.isSupported` through a compiler-derived `swiftcall` bridge, so row 105 is partial (`B`) without generic Matter or setup-readiness claims. D92 found SecureElementCredential’s Swift-only eligibility check requires the same entitlement it would need to assess and is not generic Secure Element or NFC support; row 106 remains `X`. D93 found no generic CarKey support query and its Swift API requires an MFi-limited entitlement and active Wallet vehicle session; row 107 remains `X`. D94 found no pre-existing Rust route for the Swift-only ProximityReader getter; B76 now calls only that iPhone model predicate through a compiler-derived C `swiftcall` thunk, without claiming payment-readiness. Row 108 is partial (`B`). D95 found LockedCameraCapture exposes extension session and content handoff lifecycle, not a general support or readiness query; row 109 remains `X`. D96 identified public Foundation/CoreFoundation Info.plist readers; B77 now implements only a caller-selected runtime read of `NSExtension.NSExtensionPointIdentifier`, so row 113 is partial (`B`) while build-host plist generation and `.appext` remain unsupported. D97 initially found no proven WidgetKit Rust call path; B254 now requests `WidgetCenter.shared.reloadAllTimelines()` only, so row 111 is partial (`B`) without provider, rendering, or timing guarantees. D98 matched ActivityKit compiler ABI signatures but not the owned-value link/runtime boundary, and D99 reconfirmed C7 has no documented stable App Intents metadata input or processor contract; row 110 remains `X`, and row 112 is partial (`B`) for B209. See `PLAN_CAPABILITIES_EXTENSIONKIT.md`, `PLAN_CAPABILITIES_CONTACTPROVIDER.md`, `PLAN_CAPABILITIES_BROWSERENGINEKIT.md`, `PLAN_CAPABILITIES_MANAGEDAPP.md`, `PLAN_CAPABILITIES_MARKETPLACEKIT.md`, `PLAN_CAPABILITIES_MATTERSUPPORT.md`, and `PLAN_CAPABILITIES_SECURE_ELEMENT_CREDENTIAL.md`, `PLAN_CAPABILITIES_CARKEY.md`, and `PLAN_CAPABILITIES_PROXIMITYREADER.md`, `PLAN_CAPABILITIES_LOCKED_CAMERA_CAPTURE.md`, `PLAN_CAPABILITIES_EXTENSION_BUNDLE_METADATA.md`, `PLAN_CAPABILITIES_WIDGETKIT.md`, `PLAN_CAPABILITIES_ACTIVITYKIT.md`, and `PLAN_CAPABILITIES_APP_INTENTS.md`. B76 is implemented in `platform/ios/ios-proximity-reader`; its focused gate passed and it reports only the device-model predicate. F25 now exposes B50's `VTIsHardwareDecodeSupported` predicate through the opt-in `ios-videotoolbox` C ABI. This adds no capability row or decoder-session support; device and Simulator links require minos 11.0 and 14.0, and no linked consumer/probe or live query ran. F26 now exposes B62's default video-capture-device presence through the opt-in `ios-camera-device-status` C ABI. This adds no capability row and does not query authorization, configure capture, or establish readiness; its runtime-guarded API floor is iOS 4.0, while device and Simulator probe minima are 10.0 and 14.0. No linked consumer, probe, or camera query ran. F27 now exposes B56's nonempty Core ML available-compute-device list through the opt-in `ios-core-ml-status` C ABI. This adds no capability row and does not load models, run inference, or guarantee operation support; the API floor is iOS 17.0, while measured device and Simulator link minima are 11.0 and 14.0. Static/build and native link/import gates passed; no linked consumer or probe ran.

F28 exposes B58's saved Speech authorization code through the opt-in `ios-speech-status` C ABI. It preserves signed unknown values, does not request permission or process audio, and adds no capability row. Its API floor and device link minos are iOS 10.0; Simulator minos is 14.0. The C11/C++17 link/import gates passed locally, but consumers and probes were not executed. See `PLAN_BINDINGS_F28.md`.

F29 exposes B59's English contextual-model asset state through the opt-in `ios-natural-language-status` C ABI. It maps five Rust states to fixed `uint32_t` codes; it does not load a model, accept text, calculate vectors, or request assets. The API floor and device/Simulator link minos are iOS 17.0. C11/C++17 link/import gates passed locally; consumers and probes were not executed. See `PLAN_BINDINGS_F29.md`.

F30 exposes B77's caller-selected `.appex` extension-point metadata read through the opt-in
`ios-extension-support` C ABI. Host/device/Simulator C11/C++17 link/import gates passed; the API
floor is iOS 4.0 and measured link minos are 12.0/14.0. No extension load or launch is exposed.
F31 exposes B61's `RoomCaptureSession.isSupported` result through an opt-in `ios-roomplan-status`
C ABI with an iOS 16.0 deployment floor. Its host/device/Simulator static/build and C/C++
link/import gates passed: host imports only libSystem, while device/Simulator import RoomPlan and
libSystem at minos 16.0 with the required public RoomCaptureSession symbols. Consumers and probes
were not executed. Neither binding changes capability-row coverage. See `PLAN_BINDINGS_F30.md` and
`PLAN_BINDINGS_F31.md`.

F32 exposes B63's StoreKit 2 `AppStore.canMakePayments` value through one opt-in C Boolean. Its API and
weak-symbol floor is iOS 15.0; Release C/C++ links pass with weak StoreKit and `libSystem.B.dylib` imports
and minos 10.0/14.0. The absent-symbol fallback has no runtime check below iOS 15.0. F33 exposes B55's
momentary Game Center local-player authentication value; its SDK API floor is iOS 4.1, with Foundation,
GameKit, `libSystem.B.dylib`, and `libobjc.A.dylib` imports at minos 10.0/14.0. The signed app needs the
`com.apple.developer.game-center` entitlement. Both gates pass locally; consumers were not executed, and
neither binding changes capability-row coverage. See `PLAN_BINDINGS_F32.md` and `PLAN_BINDINGS_F33.md`.

The app-data follow-up adds `IosSecurityScopedAccess` to balance one successful Foundation
security-scope start/stop pair for an already-scoped file URL; that guard itself adds no picker,
bookmark resolution, arbitrary file access, or provider lifecycle. B83 separately adds unsafe
resolution of caller-asserted plain, non-security-scoped bookmark data to a retained file URL; it
does not start security scope or alter sandbox containment. Row 014 remains partial. See
`PLAN_IOS_SECURITY_SCOPED_ACCESS.md` and `PLAN_IOS_BOOKMARK_RESOLUTION.md`.

Latest integrated checkpoint additions: B394 extends HomeKit row 072 only by counting the retained `HMAccessory.profiles` array supplied by the host; it does not claim profile readiness. B410 extends accessibility row 008 with a main-thread query for one app-registered Guided Access restriction ID; the result does not establish registration or active Guided Access and does not enforce denial. B432 declines `UIGuidedAccessConfigureAccessibilityFeatures` because it mutates system accessibility settings for Single App Mode hosts through an asynchronous callback. B431 records a descriptor-bound `_PC_CASE_SENSITIVE` feasibility GO, but no API is implemented until Darwin return and unsupported-filesystem behavior are confirmed. B401/B404/B407/B413/B416/B419/B422/B425/B428 decline volume-wide metadata without a matching app-data operation. These changes leave the capability matrix at 98/114 (`B`, 86.0%) with 16 `X`; this is row coverage, not whole-plan completion. See the focused HomeKit, accessibility, and app-data plans.

## Objective

Implement the first complete iOS backend of a reusable, high-level, platform-agnostic native application framework primarily in Rust, with:

- Rust as the direct fast path;
- a genuine `#![no_std]` portable core from the first implementation commit;
- `alloc` only where required;
- public iOS system capabilities exposed through Rust without a Swift application layer;
- public Objective-C APIs reached through `objc2`/generated bindings unless a smaller supported C path exists;
- public C/CoreFoundation/Darwin APIs called directly where preferable;
- genuinely Swift-only APIs reached through a small, capability-scoped Swift ABI subsystem with no repository-authored or generated `.swift` source;
- a stable modular C ABI for foreign-language consumers;
- optional later Python/C++ bindings without contaminating the core;
- compact/cache-conscious framework-owned state;
- future 32-bit offset/compressed-pointer compatibility without implementing pointer compression in V1;
- dependency minimization without replacing mature/security-sensitive dependencies with fragile custom code;
- Apple differential parity tests for Rust reimplementations;
- benchmark gating before any Rust implementation replaces an Apple library implementation on iOS;
- ARM64/x86-64 handwritten assembly only for measured hot paths where it is reliably faster and remains small;
- continuously updated developer and maintainer documentation.

“All APIs in Rust” means the developer-facing framework surface and application logic are Rust-native. It does **not** mean recreating Apple-owned protected databases, daemons, hardware drivers, entitlement systems, system UI, store authority, GPU/media engines, or cloud services.

## Verified repository facts

1. At planning baseline `3fa8222e6e6fff6370ac461a00fa6e61e9e84a6d`, the repository contained architecture/research docs only; no Cargo workspace or implementation crates existed. This is a historical baseline fact, not the current checkout state.
2. Most ordinary iOS capability families have public C or Objective-C routes and therefore do not require Swift ABI interoperability.
3. Swift interoperability is a residual subsystem, not the foundation.
4. The conservative V1 Swift call backend is microscopic Clang/LLVM ABI adaptation using `swiftcall` / `swiftasynccall`, with generated/compiler-oracle lowering. Nightly rustc Swift ABI support is experimental and must not be a V1 dependency.
5. Translation is the preferred first real Swift-ABI framework proof. StoreKit 2 is the stronger later forcing function.
6. Full App Intents is primarily a compiler/build metadata problem and must remain isolated from the ordinary Swift call layer.
7. Apple differential testing is required for any Rust implementation that claims Apple-compatible semantics.
8. Rust replacement of Apple/library behavior is performance-gated. Apple remains the default when equal/faster or uniquely system-integrated.
9. Portable core/public contracts must be `no_std`-compatible from the start.
10. V1 targets normal 64-bit Apple environments but must not bake 64-bit pointer width into portable semantics, stable ABI, serialized formats, IDs, or persistent indexes.
11. Framework-owned hot state should use compact layouts, narrow fields, bitsets/packed state, arenas/slabs/contiguous storage where beneficial, while keeping the public API ergonomic.
12. Dependencies must remain behind bounded internal seams and should be kept to the smallest dependable set.

## V1 definition of done

V1 is complete when all of the following are true:

### Architecture and build
- A Cargo workspace exists with independently usable portable, capability, iOS backend, Swift ABI, binding, tooling, benchmark, and example crates.
- Portable crates build as real `#![no_std]` crates with `--no-default-features`.
- iOS device and simulator builds succeed from macOS/Xcode.
- A minimal Rust-owned iOS application reaches `UIApplication -> Rust-defined delegate -> UIWindow -> UIViewController -> UILabel/UIButton -> Rust callback` without Swift application code.
- The repository contains no `.swift` source and the build generates no shipping Swift source.
- Release archive/link validation succeeds for the supported V1 packaging path.

### API and capability coverage
The framework has Rust-facing support, or an explicitly documented system/build-boundary implementation path, for the V1 capability families enumerated in `PLAN_CAPABILITIES.md`.

### Swift-only residuals
- The common Swift ABI substrate is implemented and tested.
- Translation single-string flow works end-to-end.
- StoreKit 2 product retrieval and purchase work end-to-end where StoreKit test infrastructure permits.
- Concrete StoreKit transaction update iteration is implemented if current SDK lowering can be supported without broad protocol-runtime emulation.
- Tier-1 residuals from research are implemented where they fit the common Layer-1 substrate and are available/entitled on the test host.
- App Intents build-metadata Stage 0/1 is isolated and attempted as described in `PLAN_SWIFT_ABI.md`; failure of a supported zero-Swift-source path is documented and does not contaminate the rest of the architecture.

### Correctness/performance
- Rust replacements have parity suites and benchmark evidence before becoming the iOS default.
- System-owned/hardware-accelerated Apple paths remain Apple-backed unless a measured replacement wins.
- Hot common structures have recorded size/alignment/stride/capacity invariants.
- No avoidable mandatory runtime, global executor, global service registry, serialization bridge, or universal object model exists.
- Dependency/linkage/binary-size tests exist and minimal consumers do not pull unrelated capabilities.
- No handwritten assembly is accepted without the required reference implementation, parity tests, and measured win.

### Documentation
- Public Rust APIs have useful rustdoc.
- Capability guides document availability, permissions/entitlements, lifecycle, errors, cancellation, native escape hatches, parity status, and performance behavior.
- Architecture/research docs are updated when implementation proves a different fact.
- Each significant dependency has a recorded rationale and replacement seam.

## Proposed repository layout

The exact crate names may be adjusted only if Cargo/toolchain constraints require it; dependency direction and ownership are fixed.

```text
Cargo.toml
rust-toolchain.toml
deny.toml                         # only if cargo-deny is adopted after dependency review
.clippy.toml                      # only if needed

crates/
  framework-core/
  framework-alloc/
  framework-async/
  framework-abi/
  framework-platform/
  framework-app/
  framework-data/
  framework-ui/
  framework-files/
  framework-preferences/
  framework-secure-storage/
  framework-network/
  framework-connectivity/
  framework-notifications/
  framework-location/
  framework-bluetooth/
  framework-motion/
  framework-camera/
  framework-audio/
  framework-media/
  framework-media-authorization/
  framework-photos/
  framework-contacts/
  framework-calendar/
  framework-auth/
  framework-web/
  framework-cloud/
  framework-ml/
  framework-vision/
  framework-metal/
  framework-maps/
  framework-persistence/
  framework-nfc/
  framework-health/
  framework-calling/
  framework-background/
  framework-device-integrity/
  framework-game/
  framework-vpn/
  framework-nearby/
  framework-home/
  framework-accessory/
  framework-watch-connectivity/
  framework-documents/
  framework-sharing/
  framework-payments/
  framework-apple-music/
  framework-weather/
  framework-observation/
  framework-transfer/
  framework-format/
  framework-compression/
  framework-crypto/
  framework-image/
  framework-pdf/

platform/
  ios/
    ios-runtime/
    ios-ui/
    ios-files/                          # B1 sandbox backend, B14 URLSession file adoption, B17 file coordination
    ios-preferences/
    ios-secure-storage/
    ios-network/
    ios-transfer/                       # B13 background URLSession backend; D10 portable contract integrated
    ios-notifications/
    ios-location/
    ios-bluetooth/
    ios-motion/                         # B15 one-shot Core Motion backend; D11 portable contract integrated
    ios-camera/
    ios-audio/
    ios-media/
    ios-media-authorization/
    ios-photos/
    ios-contacts/
    ios-calendar/
    ios-auth/
    ios-web/
    ios-cloud/
    ios-ml/
    ios-vision/
    ios-metal/
    ios-maps/
    ios-persistence/
    ios-nfc/
    ios-health/
    ios-calling/
    ios-background/
    ios-device-integrity/
    ios-game/
    ios-vpn/
    ios-nearby/
    ios-home/
    ios-homekit-identify-status/        # B339 host-supplied per-accessory support snapshot
    ios-accessory/
    ios-watch-connectivity/
    ios-documents/
    ios-sharing/
    ios-payments/
    ios-system-services/
    ios-extension-support/

interop/
  swift-abi-core/
  swift-abi-values/
  swift-abi-async/
  swift-abi-generated/
  swift-abi-apple/
  swift-build-metadata/

bindings/
  c/
  cpp/                             # optional V1 if cheap once C ABI exists
  python/                          # scaffold/optional; must not block core V1

tools/
  xtask/
  sdk-inventory/
  swift-oracle/
  linkage-audit/
  abi-audit/

tests/
  parity/
  integration/
  abi/
  linkage/

benchmarks/
  core/
  replacements/
  ios/

examples/
  ios-minimal/
  ios-capabilities/
  c-minimal/
```

## Shared public foundations and proposed symbols

`PLAN_FOUNDATION.md` owns these symbols. Other workstreams consume them and may not silently redefine their semantics.

### `framework-core`
Proposed public/internal foundations:

- `Platform`
- `Availability`
- `Capability`
- `CapabilityId`
- `PlatformErrorCode`
- `ErrorKind`
- `Error`
- `Result<T>`
- `Cancellation`
- `OperationId`
- `Generation`
- `CompactHandle`
- `PermissionState`
- `AuthorizationState`
- `NativeHandle<T>` only in platform-extension modules
- fixed-width time/duration primitives if needed without `std::time`

Requirements:
- `#![no_std]`
- no `alloc` unless a module proves it needs it
- no public `usize` semantic IDs
- no platform-native types
- no dependency-specific error types

### `framework-alloc`
Proposed internal reusable primitives:
- compact slab/arena primitives;
- generational handle table;
- small typed bitset/status word helpers;
- optional small-vector/storage helpers only if justified;
- hot/cold record patterns.

No public API should expose internal bit allocation.

### `framework-async`
Proposed:
- runtime-neutral `OperationState`;
- exactly-once completion primitive;
- `CancellationToken`/registration semantics;
- callback-first primitive;
- Rust `Future` adapter without a mandatory executor;
- FFI-safe completion adapter used by `framework-abi`.

### `framework-abi`
Proposed stable C-facing primitives:
- ABI version;
- fixed-width status codes;
- pointer+length slices/strings;
- owned buffer handles;
- opaque operation handles;
- cancellation;
- callback signatures;
- explicit create/destroy/retain/release functions where required;
- versioned extensible structs.

No Rust layout or third-party type may cross this boundary.

### `framework-platform`
Compile-time backend selection and platform capability markers only. No runtime registry.

## Workstream execution rule

Every named `PLAN_*.md` workstream must be executed by its own bounded subagent/executor. Parallel-safe workstreams must run concurrently in isolated branches/worktrees after prerequisites are integrated. No single agent should serially absorb multiple independent workstreams merely for convenience; if a workstream becomes too large, decompose it into additional named subplans before implementation. The orchestrator retains centralized integration and contradiction resolution.

## Workstreams

### Workstream A — Foundation
Plan: `PLAN_FOUNDATION.md`

Owns:
- root workspace/build policy;
- shared portable primitives;
- compact handles/state;
- runtime-neutral async/cancellation;
- stable internal semantic contracts;
- dependency policy implementation;
- initial docs/rustdoc conventions.

Must land first.

### Workstream B — iOS Native Runtime and Backends
Plan: `PLAN_IOS_NATIVE.md`

Depends on A.

Owns:
- objc2/C/CoreFoundation/Darwin boundary;
- Objective-C class/delegate/block machinery;
- iOS build/package bootstrap;
- capability-scoped iOS native backend crates;
- extension/system-owned service shells.

May proceed in parallel with C and D once A interfaces are integrated.

### Workstream C — Swift ABI and Compiler/Metadata Residuals
Plan: `PLAN_SWIFT_ABI.md`

Depends on A and selected tooling contracts from G.

Owns:
- Clang/LLVM Swift ABI thunk machinery;
- metadata/value witnesses;
- Swift String/Array/Optional/concrete enums/generic instances;
- async/throws/cancellation adapters;
- Translation, StoreKit 2 and other Tier-1 residual adapters;
- App Intents/build-metadata experimental subsystem;
- generated SDK ABI fixtures.

Must not modify ordinary native iOS backend ownership rules.

### Workstream D — Portable Capability Facades and Full iOS Capability Assembly
Plan: `PLAN_CAPABILITIES.md`

Depends on A; each capability implementation consumes B or C backend pieces.

Owns:
- developer-facing high-level capability crates;
- semantic portable contracts;
- platform extensions;
- capability-specific rustdoc/guides;
- full capability coverage matrix.

Capability modules may be implemented in parallel after their shared contracts are approved. D15
is the separate informational network-path snapshot contract in
`PLAN_CAPABILITIES_CONNECTIVITY.md`; B18 implements its iOS backend in
`PLAN_IOS_CONNECTIVITY.md`. It does not expand D1/B3 HTTP or promise endpoint reachability.
D19 adds a bounded outbound TLS-over-TCP byte-stream contract in
`PLAN_CAPABILITIES_CONNECTION.md`, and B24 implements it through Network.framework C APIs in
`PLAN_IOS_CONNECTION.md`; listeners, UDP, and endpoint preflight remain out of scope. D20 adds a
synchronous app-refresh contract in `PLAN_CAPABILITIES_BACKGROUND_TASKS.md`, and B25 implements
one `BGAppRefreshTask` path in `PLAN_IOS_BACKGROUND_TASKS.md`; scheduling remains OS-controlled,
and host task identifiers and background-mode metadata remain app-owned. D21/B26 adds portable
image-source dimensions/count and a metadata-only ImageIO path; D22/B27 adds Photos read/write
authorization only, with `Limited` distinct from `Authorized`. D23/B28 adds one UIKit
background-execution lease with cooperative expiry and explicit end; it does not promise extra
runtime, future launch, or work completion, and app extensions are unsupported. D24/B29 adds Contacts
authorization status and request only, preserving `Limited` separately from full access and leaving
enumeration, fetch, edits, and picker UI out of scope. D25/B30 adds EventKit Calendar event-authorization
status and an explicit full-access request for iOS 17.0+, preserving `WriteOnly` separately from
`FullAccess` and excluding event/reminder data operations. These bounded slices do not claim full
platform parity.

D28/B33 adds a platform-exclusive HTTPS URL contract and a bounded typed `WKWebView` adapter for
navigation controls. It does not add browser parity, JavaScript, arbitrary file/HTML/data loading,
subresource confinement, or a page-load guarantee. See `PLAN_CAPABILITIES_WEB.md` and
`PLAN_IOS_WEB.md`.

D29/B34 adds explicit unfiltered foreground central discovery, copied peer UUID/RSSI values, and a
fixed 32-event queue; the scan may prompt, starts asynchronously, and has no background-delivery
guarantee. No connection, advertisement, peripheral operations, or radio control are included. See
`PLAN_CAPABILITIES_BLUETOOTH.md`, `PLAN_IOS_BLUETOOTH.md`, and
`PLAN_IOS_BLUETOOTH_DISCOVERY.md`.

D30/B35 adds a presence-only snapshot of
`NSFileManager.ubiquityIdentityToken`; it does not expose the token or prove container access,
synchronization, or CloudKit account status. See `PLAN_CAPABILITIES_ICLOUD_DRIVE_IDENTITY.md` and
`PLAN_IOS_ICLOUD_DRIVE_IDENTITY.md`.

D47/B52 exposes one owned CloudKit account-status snapshot from the app's default `CKContainer`. It
does not return account identity, access records or databases, observe account changes, or prove
configured container access. It requires the signed app's CloudKit container/service entitlements;
dropping its Rust future abandons interest but cannot cancel the native query. See
`PLAN_CAPABILITIES_CLOUDKIT_ACCOUNT_STATUS.md` and `PLAN_IOS_CLOUDKIT_ACCOUNT_STATUS.md`.

D48/B53 exposes only SafetyKit's point-in-time Crash Detection device-support bit; it does not
claim app entitlement or authorization, receive crash events, or provide emergency response. The
getter-specific entitlement prerequisite remains unverified. D49/B54 exposes only the system's
support result for an ARKit world-tracking configuration; it does not create a session, access the
camera, request camera permission, or establish active tracking. D50/B55 exposes only a synchronous
Game Center local-player authentication snapshot; it does not initialize authentication, show UI,
observe status changes, or expose identity/game data. See the D48–D77 and B53–B73 plans for exact
scope and evidence limits.

D51/B56 exposes only whether Core ML's `MLModel.availableComputeDevices` list is nonempty on iOS
17.0+. It loads no model, runs no inference, returns no compute-device objects, and does not
guarantee that any particular model or operation can run. See
`PLAN_CAPABILITIES_COREML.md` and `PLAN_IOS_COREML.md`.

D52/B57 exposes whether a caller-supplied revision is present in
`VNRecognizeTextRequest.supportedRevisions` from iOS 13.0. It creates no request, reads no image,
and does not establish recognition success or model readiness. See `PLAN_CAPABILITIES_VISION.md`
and `PLAN_IOS_VISION.md`.

D53/B58 reads only the app's saved Speech authorization status through
`SFSpeechRecognizer::authorizationStatus` from iOS 10.0. It does not request permission, create a
recognizer, accept audio, or start recognition; authorized status does not prove service availability
or recognition success. See `PLAN_CAPABILITIES_SPEECH_STATUS.md` and
`PLAN_IOS_SPEECH_STATUS.md`.

D54/B59 checks only whether Apple’s built-in English contextual-model assets are on-device through
`NLContextualEmbedding::hasAvailableAssets` from iOS 17.0. It does not load a model, accept text,
compute vectors, request assets, or guarantee a later model operation. See
`PLAN_CAPABILITIES_NATURALLANGUAGE_STATUS.md` and `PLAN_IOS_NATURALLANGUAGE_STATUS.md`.

D55/B60 reads only the deprecated StoreKit 1 `SKPaymentQueue::canMakePayments` bit. It does not
create a queue, inspect a product or account, show payment UI, or process a transaction. See
`PLAN_CAPABILITIES_STOREKIT_STATUS.md` and `PLAN_IOS_STOREKIT_STATUS.md`.

D56/B61 reads only `RoomCaptureSession.isSupported` from iOS 16.0 through a compiler-verified
`swiftcall` thunk. It does not create a session, access camera or LiDAR frames, request permission, or
claim scan success. See `PLAN_CAPABILITIES_ROOMPLAN.md` and `PLAN_IOS_ROOMPLAN.md`.

D57/B62 adds only an iOS 4.0+ default-video-device presence query through
`AVCaptureDevice.defaultDeviceWithMediaType(AVMediaTypeVideo)`. It does not query authorization,
create a capture input/session, access media, or present UI. See
`PLAN_CAPABILITIES_CAMERA_DEVICE_STATUS.md` and `PLAN_IOS_CAMERA_DEVICE_STATUS.md`.

D58/B63 adds only the iOS 15.0+ `AppStore.canMakePayments` status through a compiler-verified
weak-import `swiftcall` thunk. It does not retrieve products, access account/entitlement data, show
purchase UI, or process transactions. See `PLAN_CAPABILITIES_STOREKIT2_STATUS.md` and
`PLAN_IOS_STOREKIT2_STATUS.md`.

D60/B66 adds only iOS 2.0+ one-shot SHA-256 through Apple's CommonCrypto `CC_SHA256`. Its safe Rust
wrapper bounds input to the `CC_LONG` width and returns an owned 32-byte digest; there is no
portable crypto contract, key operation, replacement/parity, or performance claim. See
`PLAN_CAPABILITIES_CRYPTO.md`, `PLAN_IOS_CRYPTO.md`, and `PLAN_VALIDATION_IOS_CRYPTO.md`.

D59/B65 adds only iOS 4.0+ single-precision vector addition through `vDSP_vadd`. The safe Rust
wrapper requires equal input/output lengths, uses unit strides, and returns without a native call
for empty slices. It adds no portable contract, other Accelerate operations, performance claim, or
bitwise parity claim. See `PLAN_CAPABILITIES_ACCELERATE.md` and `PLAN_IOS_ACCELERATE.md`.

B64 adds an HTTPS-only `SFSafariViewController` constructor using `SafariServices`; the host owns
UIKit presentation and dismissal. It does not expose browser data, start an external URL handler,
or claim a request/page load. See `PLAN_IOS_SAFARI.md`.

D61/B67 adds only the iOS ModelIO extension-level `MDLAsset.canImportFileExtension` query. It does
not access a URL or file data, parse an asset, render content, or claim GPU support. See
`PLAN_CAPABILITIES_MODELIO_STATUS.md`, `PLAN_IOS_MODELIO_STATUS.md`, and
`PLAN_VALIDATION_IOS_MODELIO_STATUS.md`.

D62/B68 adds only whether the default-options `MPSGetPreferredDevice` call returns a device on
iOS 12.2+. It submits no GPU work and does not establish support for any MPS operation, model, or
workload. See `PLAN_CAPABILITIES_MPS_STATUS.md`, `PLAN_IOS_MPS_STATUS.md`, and
`PLAN_VALIDATION_IOS_MPS_STATUS.md`.

D63/B69 adds a portable borrowed value for one uncompressed P-256 public key and an iOS Security
query for ECDSA/SHA-256 message-verification suitability. It does not verify a signature, use a
private key, persist a key, or access the Secure Enclave. See `PLAN_CAPABILITIES_KEY_SUPPORT.md`,
`PLAN_IOS_KEY_SUPPORT.md`, and `PLAN_VALIDATION_IOS_KEY_SUPPORT.md`.

D64/B70 adds a portable finite `SpriteNodePosition` contract and a detached iOS `SKNode` create/get/set facade for parent-local `position` only. The public API floor is iOS 7.0; local device/Simulator link-probe minimums are 12.0/14.0. It does not add SceneKit, a scene/view/render loop, hierarchy, animation, physics, or assets. See `PLAN_CAPABILITIES_SPRITEKIT.md`, `PLAN_IOS_SPRITEKIT.md`, and `PLAN_VALIDATION_IOS_SPRITEKIT.md`.

D65/B71 reads the current MediaPlayer library authorization status on iOS 9.3+. It does not request
access, read media items, contact Apple Music services, or provide catalog/playback support. The
host must supply `NSAppleMusicUsageDescription` before it requests library access or reads items;
this status-only query does neither. See `PLAN_CAPABILITIES_MEDIA_LIBRARY_STATUS.md`,
`PLAN_IOS_MEDIA_LIBRARY_STATUS.md`, and `PLAN_VALIDATION_IOS_MEDIA_LIBRARY_STATUS.md`.

D76/B72 reads a synchronous CallKit call snapshot and copies only count plus aggregate state flags;
it exposes no call objects, identifiers, caller data, callbacks, call control, provider, PushKit, or
audio API. The initial native read may block. See `PLAN_CAPABILITIES_CALLKIT.md` and
`PLAN_VALIDATION_IOS_CALL_OBSERVER.md`.

D77/B73 adds finite portable map coordinates, map points, and distances plus MapKit conversion and
distance calls. It does not add map UI, location, permissions, network service, search, or directions.
See `PLAN_CAPABILITIES_MAPKIT.md` and `PLAN_VALIDATION_IOS_MAPKIT.md`.

D81/B74 reads only the caller-owned `NSUserActivity.isClassKitDeepLink` Boolean on iOS 11.3+; it
does not access the ClassKit store or assignment data. Schoolwork host adoption and the separate
ClassKit environment entitlement remain outside this query. See `PLAN_CAPABILITIES_CLASSKIT.md` and `PLAN_VALIDATION_IOS_CLASSKIT.md`.

D67/B80 implements a raw signed iOS 15+ `AuthorizationCenter.shared.authorizationStatus` snapshot through a compiler-matched C `swiftcall` bridge. Its unsafe API requires the main dispatch queue and makes no entitlement, distribution approval, activity-data access, or control-use claim. Row 074 is `B`/partial; row 075 remains separate and `X`. See `PLAN_CAPABILITIES_FAMILY_CONTROLS_STATUS.md`.

D69 is a feasibility audit for row 067 Foundation Models. Its smallest status candidate is Swift-only and has no public C/Objective-C entry point or generated Rust binding; row 067 remains `X` until a supported wrapper boundary is integrated. See `PLAN_CAPABILITIES_FOUNDATION_MODELS.md`.

D71 is a feasibility audit for row 075 DeviceActivity/ManagedSettings. A public iOS 17 Objective-C authorization getter is callable in principle, but Apple does not define its Boolean meaning, prompt behavior, thread guarantees, or query-only entitlement requirements; row 075 remains `X`. See `PLAN_CAPABILITIES_DEVICE_ACTIVITY.md`.

D31/B36 adds camera and microphone authorization-status queries only; it does not request access,
create capture/audio sessions, select devices, or access samples. The API floor is iOS 7.0; usage
description keys apply before host access requests or capture. See
`PLAN_CAPABILITIES_MEDIA_AUTHORIZATION.md` and `PLAN_IOS_MEDIA_AUTHORIZATION.md`.

D32/B37 adds the Core NFC `readingAvailable` support snapshot only, with an iOS 11.0 API floor; no
session or tag operation is included. `NFCReaderUsageDescription` is conservatively recorded
because Apple does not state a property-only exception. See `PLAN_CAPABILITIES_NFC.md` and
`PLAN_IOS_NFC.md`.

D33's HomeKit status-only audit found that `HMHomeManager.authorizationStatus` requires an instance
and first HomeKit use can prompt; no non-prompting static query or separate request API exists. Row
072 remains `X` for the status-only scope.D33/B173's HomeKit status-only audit found that `HMHomeManager.authorizationStatus` requires an instance
and first HomeKit use can prompt; no non-prompting static query or separate request API exists. Row
072 remains `X` for the status-only scope ([B173 audit](PLAN_CAPABILITIES_HOMEKIT.md)).

D34/B39 adds one iOS 16.0+ Nearby Interaction device-capability query. It reports only
`supportsPreciseDistanceMeasurement`; it does not create a session, request permission, exchange
tokens, discover peers, or start ranging. See `PLAN_CAPABILITIES_NEARBY_INTERACTION.md` and
`PLAN_IOS_NEARBY_INTERACTION.md`.

D35/B40 adds App Tracking Transparency status only through `ATTrackingManager` on iOS 14.0+.
It does not request authorization, access IDFA, or represent other privacy permissions. See
`PLAN_CAPABILITIES_PRIVACY_AUTHORIZATION.md` and `PLAN_IOS_TRACKING_AUTHORIZATION.md`.

D36/B41 adds only a scalar report that `MTLCreateSystemDefaultDevice` returned a device object on
iOS 8.0+. It drops the retained object and submits no GPU work; MetalKit, rendering, compute,
features, and performance remain out of scope. See `PLAN_CAPABILITIES_METAL.md` and
`PLAN_IOS_METAL.md`.

D37/B42 reads only DeviceCheck support from iOS 11.0 and App Attest support from iOS 14.0. It does
not create keys/tokens, attest/assert, contact a server, or establish integrity. D38/B43 reads only
whether iOS 9.0+ can provide a Watch Connectivity session object; it does not inspect pairing or
communicate. D39/B44 reports only whether the iOS ExternalAccessory connected-and-available list
is empty at query time; it does not open a session or establish hardware communication. D40/B45 reads
legacy ReplayKit availability only and does not start capture or recording; Apple marks this query
deprecated and recommends ScreenCaptureKit. D41/B46 checks only whether the built-in SoundAnalysis classifier request is recognized; it does not analyze audio or establish microphone access. D42/B47 reads only the iOS other-audio Boolean; it does not identify the source, report this app's playback, or control media. D43/B48 adds only a main-thread, iOS 13.4+ `AVPlayer.eligibleForHDRPlayback` snapshot; it does not inspect an asset or start playback. D44/B49 adds `canSendMail`, `canSendText`, and the SharedWithYou software-support bit only; it does not create, present, or send messages or read collaboration/account data. D45/B50 adds only the VideoToolbox hardware-decode support predicate for a caller codec; it does not encode, create sessions, or process media frames. D46/B51 adds only PassKit general Apple Pay device capability; it does not inspect cards, merchant eligibility, or process a payment. D47/B52 adds one CloudKit account-status snapshot only; it does not access data or observe account changes. D48/B53 reads SafetyKit Crash Detection device support only; D49/B54 queries ARKit world-tracking configuration support only; D50/B55 reads Game Center's local-player authentication Boolean only; D51/B56 checks only for a nonempty Core ML compute-device list; D52/B57 checks only for a listed Vision text-recognition revision; D53/B58 reads saved Speech authorization status only and does not request permission or process audio; D54/B59 checks only the asset state of Apple’s built-in English contextual model and does not load a model or accept text. See the D37–D59 and B42–B65 plans for exact scope.

### Workstream E — Rust Replacement Candidates
Plan: `PLAN_REPLACEMENTS.md`

Depends on A and G benchmark/parity harness. May run parallel with B/C/D.

Owns:
- pure Rust candidate implementations for library-like work;
- differential Apple reference adapters used only in tests;
- selection gates deciding Rust vs Apple default per operation.

Must not reimplement system-owned/hardware-accelerated services simply to remove Apple dependencies.

### Workstream F — Stable Foreign-Language Bindings
Plan: `PLAN_BINDINGS.md`

Depends on A and stable capability contracts from D.

Owns:
- stable C API;
- generated/maintained C headers;
- C++ convenience layer if feasible without new runtime cost;
- optional Python binding scaffold after C/Rust semantics are stable.

Rust-native API must never route through this layer.

### Workstream G — Validation, Tooling, Performance and Documentation Infrastructure
Plan: `PLAN_VALIDATION.md`

Begins after A establishes crate names; thereafter runs alongside all workstreams.

Owns:
- xtask/tooling;
- SDK inventory;
- Swift oracle generation;
- parity harness framework;
- linkage/ABI audit tooling;
- benchmark harness;
- no_std/dependency/binary-size/codegen gates;
- global documentation indexes and final V1 audit.

Individual workstreams own tests/docs next to their code; G owns shared harnesses and cross-workstream validation.

## Dependency graph and parallel safety

```text
A Foundation
|
+----> B iOS Native ------------------+
|                                     |
+----> C Swift ABI -------------------+----> D capability assembly
|                                     |          |
+----> G validation/tooling ----------+          |
|                                                v
+-----------------------------> E replacements --+--> F foreign bindings
                                                  |
                                                  v
                                             final integration
```

Parallel-safety rules:

- A is integrated first.
- After A, B/C/G can proceed in isolated branches/worktrees.
- D capability crates may begin when the relevant A contract is stable; each capability waits only for the backend primitives it needs.
- E can prototype pure Rust replacements as soon as G provides benchmark/parity harness contracts.
- F starts only after the relevant developer-facing capability contract is stable enough to freeze a C ABI.
- No workstream may modify another workstream's owned shared symbols without central reconciliation.
- Root `Cargo.toml` should use workspace glob patterns established by A so later workstreams can add crates without editing the root member list.
- Shared dependency versions/features are centralized in workspace dependencies by A; later additions require explicit dependency rationale and conflict review.

## Capability implementation classification

For every capability, executor must record one of:

- **R** — pure/portable Rust implementation selected after parity/performance proof;
- **M** — Rust semantics/state machine over a minimal public Apple primitive;
- **B** — Apple/system-owned backend reached through Rust;
- **A** — Apple implementation preferred for performance/hardware reasons;
- **C** — compiler/build/discovery contract requiring packaging metadata;
- **X** — unsupported/deferred because a public zero-Swift-source path is not yet proven.

A capability may have multiple labels (for example H/A behavior is represented as Rust facade + Apple accelerated backend).

## Global invariants

1. No `.swift` files in repository or generated shipping sources.
2. No private frameworks, selectors, daemons, symbols, entitlements, or App Review bypasses.
3. Portable crates are `no_std` from their first implementation.
4. No mandatory Tokio/async-std or universal executor.
5. No process-global `Framework::initialize()`.
6. No universal dependency-injection/service-locator/runtime registry.
7. No virtual DOM or duplicate platform UI object model.
8. No C ABI round-trip for Rust callers.
9. No dependency-specific types in portable public APIs.
10. No speculative assembly.
11. No 64-bit pointer-width semantic assumptions.
12. No performance replacement without parity/correctness first.
13. No fragile in-house crypto/ABI/parser replacement solely to lower dependency count.
14. Panics never unwind across FFI.
15. Objective-C/Swift ownership follows native ownership; do not wrap every native object in `Arc`.
16. Main-thread restrictions are typed/platform-specific.
17. Async operations have explicit cancellation and exactly-once completion.
18. Public API remains semantic/ergonomic; packed layouts stay internal.

## Integration order

1. Integrate A.
2. Integrate G bootstrap/no_std/toolchain/linkage/parity harness.
3. Integrate B native iOS runtime and minimal app vertical slice.
4. Integrate C Swift ABI synchronous foundation, then async foundation.
5. Integrate D capabilities in batches, using B/C backends.
6. Integrate E only for candidates that pass parity + performance gates.
7. Integrate F after capability contracts stabilize.
8. Run complete cross-workstream validation.
9. Independently compare final diff against this PLAN.md and every workstream plan.
10. Resolve contradictions centrally; do not let a workstream silently redesign architecture.
11. Delete all `PLAN*.md` files before final implementation commit/release branch merge.

## Cross-workstream validation

Minimum commands/activities once implemented:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo check -p framework-core --no-default-features
cargo check -p framework-alloc --no-default-features
cargo check -p framework-async --no-default-features
cargo check -p framework-abi --no-default-features
cargo tree --workspace
cargo tree -d
cargo doc --workspace --no-deps
```

Plus macOS/Xcode tasks provided by `tools/xtask`:

```text
xtask toolchain-manifest
xtask ios-build --simulator --release
xtask ios-build --device --release
xtask ios-minimal-link-audit
xtask swift-abi-oracles
xtask parity --ios
xtask abi-audit
xtask dependency-audit
xtask size-audit
xtask codegen-audit
xtask archive-smoke
```

Exact CLI spelling may change during implementation, but equivalent functionality is required.

## Final diff checklist

Before V1 is accepted:

- [x] No `.swift` source exists.
- [x] Portable crates genuinely compile without `std`.
- [ ] No portable API exposes platform/dependency types.
- [ ] No new mandatory runtime/global registry/executor.
- [ ] Capability crates remain independently linkable.
- [ ] Minimal consumers do not pull unrelated frameworks.
- [ ] Public C ABI contains no Rust layout.
- [ ] Panics cannot cross FFI.
- [ ] Objective-C/Swift retain/release/destroy paths are stress-tested.
- [ ] Async cancellation/completion races are deterministic and tested.
- [ ] Rust replacements have Apple parity fixtures.
- [ ] Rust replacements selected by default have benchmark evidence.
- [ ] Hardware/system-owned Apple capabilities remain Apple-backed unless a measured exception exists.
- [ ] Handwritten assembly has portable reference, differential tests, CPU/ABI gating, and meaningful measured win.
- [ ] Hot structures have size/alignment/capacity records.
- [ ] Dependency additions have rationale and minimal features.
- [ ] No public semantic ID/serialized field accidentally depends on `usize`.
- [ ] iOS simulator/device builds pass.
- [ ] Release archive/link smoke passes.
- [ ] Developer docs and capability support matrix are current.
- [ ] Every executor reports changed files, commit SHA, tests, deviations, unresolved assumptions.
- [ ] PLAN files are removed before the final implementation commit.

## Executor handoff

The orchestrator should start implementation with:

> Implement `PLAN.md` exactly. Verify latest `main` first. Read the assigned `PLAN_*.md` before coding. Stay within scope unless current code, compilation, tests, moved symbols, compatibility, or correctness require expansion. Do not silently redesign shared architecture. Run the required tests, inspect the diff, report changed files/commit SHA/tests/deviations/unresolved assumptions, and let the orchestrator integrate in PLAN.md order. After final independent validation, delete all PLAN*.md files before the implementation is finalized.
