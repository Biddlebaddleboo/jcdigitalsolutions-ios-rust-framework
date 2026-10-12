import json
from pathlib import Path
import re
import sys


ADAPTER_ID = "ios-app-lifecycle-contract"
CAPABILITY_ID = "001-core-app-ui-application-lifecycle"
PACKAGE_ROOT = "platform/ios/ios-app-lifecycle"
LIB_PATH = PACKAGE_ROOT + "/src/lib.rs"
PLATFORM_PATH = PACKAGE_ROOT + "/src/platform.rs"
QUEUE_PATH = PACKAGE_ROOT + "/src/queue.rs"
MAPPING_PATH = PACKAGE_ROOT + "/src/mapping.rs"
OBSERVERS_PATH = PACKAGE_ROOT + "/src/observers.rs"
CARGO_PATH = PACKAGE_ROOT + "/Cargo.toml"
CHECK_PATH = PACKAGE_ROOT + "/check.sh"
LINK_CHECK_PATH = PACKAGE_ROOT + "/check-link-imports.sh"
LINK_PROBE_PATH = PACKAGE_ROOT + "/examples/link_probe.rs"
RUNTIME_ROOT = "platform/ios/ios-runtime"
RUNTIME_MANIFEST_PATH = RUNTIME_ROOT + "/Cargo.toml"
RUNTIME_LIB_PATH = RUNTIME_ROOT + "/src/lib.rs"
DISPATCH_PATH = RUNTIME_ROOT + "/src/dispatch.rs"
MAIN_THREAD_PATH = RUNTIME_ROOT + "/src/main_thread.rs"
EXPECTED_TARGETS = {
 ("aarch64-apple-ios", "iphoneos"): "10.0",
 ("aarch64-apple-ios-sim", "iphonesimulator"): "14.0",
}
DOC_PATH = "docs/ios/app-lifecycle.md"
MANIFEST_PATH = "docs/capabilities/capability-status.json"
SPEC_PATH = "tools/validation/specs/validation-v1.json"
REQUIRED_INPUTS = {
 LIB_PATH,
 PLATFORM_PATH,
 QUEUE_PATH,
 MAPPING_PATH,
 OBSERVERS_PATH,
 CARGO_PATH,
 CHECK_PATH,
 LINK_CHECK_PATH,
 LINK_PROBE_PATH,
 RUNTIME_MANIFEST_PATH,
 RUNTIME_LIB_PATH,
 DISPATCH_PATH,
 MAIN_THREAD_PATH,
 DOC_PATH,
 MANIFEST_PATH,
 SPEC_PATH,
}
NOTIFICATION_MAP = {
 "UIApplicationDidBecomeActiveNotification": "DidBecomeActive",
 "UIApplicationWillResignActiveNotification": "WillResignActive",
 "UIApplicationWillEnterForegroundNotification": "WillEnterForeground",
 "UIApplicationDidEnterBackgroundNotification": "DidEnterBackground",
 "UIApplicationWillTerminateNotification": "WillTerminate",
}
PRIVATE_OR_DELEGATE_PATTERNS = [
 re.compile(pattern)
 for pattern in (
  r"\bsetDelegate\s*\(",
  r"\bset_delegate\s*\(",
  r"\bperformSelector\s*\(",
  r"\bvalueForKey\s*\(",
  r"\b(?:dlsym|objc_getClass|objc_lookUpClass|NSClassFromString|NSSelectorFromString)\s*\(",
  r"\b(?:LSApplicationWorkspace|SpringBoard|UIApplicationPrivate)\b",
  r"\b(?:swift_abi|SwiftABI)\b",
 )
]


def emit(status, detail):
 print(json.dumps({"schema_version": 1, "status": status, "detail": detail}, separators=(",", ":")))


def read_inputs(payload):
 if payload.get("schema_version") != 1 or payload.get("id") != ADAPTER_ID:
  raise ValueError("invalid adapter protocol version or id")
 paths = payload.get("input_paths")
 if not isinstance(paths, list) or any(not isinstance(path, str) for path in paths):
  raise ValueError("input_paths must be a list of repository-relative strings")
 if len(paths) > 16 or len(paths) != len(set(paths)) or not REQUIRED_INPUTS.issubset(set(paths)):
  raise ValueError("required iOS lifecycle source, docs, or manifest input is missing")
 root = Path(payload["workspace_root"]).resolve(strict=True)
 sources = {}
 total_bytes = 0
 for relative in paths:
  candidate_path = Path(relative)
  if candidate_path.is_absolute() or ".." in candidate_path.parts:
   raise ValueError("input path is not workspace-relative")
  source_path = (root / candidate_path).resolve(strict=True)
  if root not in source_path.parents or not source_path.is_file():
   raise ValueError("input path escaped workspace root or is not a file")
  size = source_path.stat().st_size
  total_bytes += size
  if size > 1_048_576 or total_bytes > 2_097_152:
   raise ValueError("adapter input exceeds the source-size budget")
  sources[relative] = source_path.read_text(encoding="utf-8")
 return root, sources


def rust_without_comments(source):
 source = re.sub(r"/\*.*?\*/", " ", source, flags=re.S)
 return re.sub(r"(?m)//[^\n]*", " ", source)


def has_swift_source(root):
 package = (root / PACKAGE_ROOT).resolve(strict=True)
 if root not in package.parents or not package.is_dir():
  raise ValueError("iOS lifecycle package directory is missing or invalid")
 inspected = 0
 for path in package.rglob("*"):
  inspected += 1
  if inspected > 512:
   raise ValueError("iOS lifecycle package exceeds the bounded source scan")
  if path.suffix == ".swift":
   return True
 return False


def near(text, first, second, distance=500):
 start = 0
 while True:
  index = text.find(first, start)
  if index < 0:
   return False
  if second in text[index:index + distance]:
   return True
  start = index + len(first)


def rust_function_body(source, name):
 match = re.search(r"\bpub\s+fn\s+" + re.escape(name) + r"\s*\(", rust_without_comments(source))
 if match is None:
  return None
 clean = rust_without_comments(source)
 opening = clean.find("{", match.end())
 if opening < 0:
  return None
 depth = 0
 for index in range(opening, len(clean)):
  if clean[index] == "{":
   depth += 1
  elif clean[index] == "}":
   depth -= 1
   if depth == 0:
    return clean[match.start():index + 1]
 return None


def drop_removes_observer(source):
 match = re.search(r"\bimpl(?:<[^>]*>)?\s+Drop\s+for\s+\w+", source)
 if match is None:
  return False
 opening = source.find("{", match.end())
 if opening < 0:
  return False
 depth = 0
 for index in range(opening, len(source)):
  if source[index] == "{":
   depth += 1
  elif source[index] == "}":
   depth -= 1
   if depth == 0:
    return re.search(r"\bremove\w*\s*\(", source[opening + 1:index]) is not None
 return False


def check_unknown_state_error(mapping):
 raw_code = re.search(r"PlatformErrorCode::new\s*\(\s*([A-Za-z_]\w*)\s*\)", mapping)
 if raw_code is None:
  return False
 raw_name = re.escape(raw_code.group(1))
 fallback = re.search(r"(?s)\b" + raw_name + r"\s*=>.*?ErrorKind::Platform.*?\.with_platform_code\s*\(.*?PlatformErrorCode::new\s*\(\s*" + raw_name + r"\s*\)", mapping)
 return fallback is not None


def check_profile_targets(spec_text):
 spec = json.loads(spec_text)
 pilot = next((item for item in spec.get("pilots", []) if item.get("id") == "ios-app-lifecycle"), None)
 if pilot is None or pilot.get("package") != "ios-app-lifecycle":
  return False
 for key in ("targets", "abi_targets"):
  targets = pilot.get(key, [])
  for (triple, sdk), minimum in EXPECTED_TARGETS.items():
   matches = [target for target in targets if target.get("triple") == triple and target.get("sdk") == sdk]
   if len(matches) != 1 or matches[0].get("minimum_os") != minimum:
    return False
 if LINK_CHECK_PATH not in pilot.get("abi_scripts", []):
  return False
 if LINK_CHECK_PATH not in pilot.get("shell_scripts", []):
  return False
 return True


def check_profile_floor_regressions(spec_text):
 if not check_profile_targets(spec_text):
  return False
 for key in ("targets", "abi_targets"):
  for triple, sdk in EXPECTED_TARGETS:
   mutated = json.loads(spec_text)
   pilot = next(item for item in mutated["pilots"] if item.get("id") == "ios-app-lifecycle")
   matches = [target for target in pilot[key] if target.get("triple") == triple and target.get("sdk") == sdk]
   if len(matches) != 1:
    return False
   matches[0]["minimum_os"] = "0.0"
   if check_profile_targets(json.dumps(mutated)):
    return False
 return True


def check_dispatch_contract(sources):
 runtime_manifest = sources[RUNTIME_MANIFEST_PATH]
 lifecycle_manifest = sources[CARGO_PATH]
 runtime_lib = sources[RUNTIME_LIB_PATH]
 dispatch = " ".join(rust_without_comments(sources[DISPATCH_PATH]).split())
 main_thread = rust_without_comments(sources[MAIN_THREAD_PATH])
 link_probe = " ".join(rust_without_comments(sources[LINK_PROBE_PATH]).split())
 link_script = sources[LINK_CHECK_PATH]
 features = re.search(r"(?ms)^\[features\]\s*(.*?)(?=^\[|\Z)", runtime_manifest)
 dependency = re.search(r"(?ms)^dispatch2\s*=\s*\{([^}]*)\}", runtime_manifest)
 if features is None or dependency is None:
  return False
 if not re.search(r'(?m)^dispatch\s*=\s*\["dep:dispatch2"\]\s*$', features.group(1)):
  return False
 if re.search(r'(?m)^default\s*=\s*\[[^\]]*"dispatch"', features.group(1)):
  return False
 if not re.search(r"\boptional\s*=\s*true\b", dependency.group(1)):
  return False
 if not re.search(r'(?m)^main-thread-dispatch\s*=\s*\["ios-runtime/dispatch"\]\s*$', lifecycle_manifest):
  return False
 if not re.search(r'(?m)#\[cfg\(feature\s*=\s*"dispatch"\)\]\s*pub\s+mod\s+dispatch\s*;', runtime_lib):
  return False
 if not re.search(r"(?m)#\[cfg\(feature\s*=\s*\"dispatch\"\)\]\s*pub\(crate\)\s+unsafe\s+fn\s+assume_current", main_thread):
  return False
 if "pub fn run_on_main(callback: impl FnOnce(MainThread) + Send + 'static) -> MainThreadHandoff" not in dispatch:
  return False
 if not all(fragment in dispatch for fragment in (
  "MainThread::current()",
  "callback(main_thread)",
  "DispatchQueue::main().exec_async",
  "MainThread::assume_current()",
  "catch_unwind(|| callback(main_thread))",
  "MainThreadHandoff::RanInline",
  "MainThreadHandoff::Enqueued",
 )):
  return False
 if not re.search(r"(?s)#\[cfg\(feature\s*=\s*\"main-thread-dispatch\"\)\].*?run_on_main\(", link_probe):
  return False
 return all(fragment in link_script for fragment in (
  "--features main-thread-dispatch",
  "libSystem.B.dylib",
  "_dispatch_(get_main_queue|async_f)",
 ))


def check_dispatch_negative_regressions(sources):
 cases = (
  (RUNTIME_MANIFEST_PATH, 'dispatch = ["dep:dispatch2"]', 'default = ["dispatch"]\ndispatch = ["dep:dispatch2"]'),
  (RUNTIME_MANIFEST_PATH, 'dispatch2 = { workspace = true, features = ["alloc"], optional = true }', 'dispatch2 = { workspace = true, features = ["alloc"], optional = false }'),
  (MAIN_THREAD_PATH, "pub(crate) unsafe fn assume_current", "pub unsafe fn assume_current"),
  (DISPATCH_PATH, "FnOnce(MainThread) + Send", "FnOnce(MainThread)"),
  (LINK_CHECK_PATH, "--features main-thread-dispatch", "--features missing-dispatch"),
  (LINK_PROBE_PATH, '#[cfg(feature = "main-thread-dispatch")]\n        let _ = ios_runtime::dispatch::run_on_main', '#[cfg(feature = "main-thread-dispatch")]\n        let _ = ios_runtime::dispatch::missing_dispatch'),
 )
 for path, before, after in cases:
  if before not in sources[path]:
   return False
  mutated = dict(sources)
  mutated[path] = sources[path].replace(before, after, 1)
  if check_dispatch_contract(mutated):
   return False
 return True


def check_application_uri_contract(sources):
 library = rust_without_comments(sources[LIB_PATH])
 platform = rust_without_comments(sources[PLATFORM_PATH])
 body = rust_function_body(sources[PLATFORM_PATH], "open_application_uri")
 if body is None:
  return False
 compact_body = " ".join(body.split())
 link_probe = sources[LINK_PROBE_PATH]
 docs = " ".join(sources[DOC_PATH].split()).lower()
 if not re.search(r"pub\s+use\s+platform::\s*\{[^}]*OpenApplicationUriError[^}]*open_application_uri", library, re.S):
  return False
 if not all(fragment in platform for fragment in (
  "pub enum OpenApplicationUriError",
  "FoundationRejectedUri",
  "MissingScheme",
 )):
  return False
 if not all(fragment in compact_body for fragment in (
  "main_thread: MainThread",
  "uri: &str",
  "completion: impl FnOnce(bool) + 'static",
  "core::result::Result<(), OpenApplicationUriError>",
  "NSString::from_str(uri)",
  "NSURL::URLWithString(&text)",
  "return Err(OpenApplicationUriError::FoundationRejectedUri)",
  "url.scheme().is_none_or(|scheme| scheme.length() == 0)",
  "return Err(OpenApplicationUriError::MissingScheme)",
  "Rc::new(RefCell::new(Some(completion)))",
  "ios_runtime::ffi::catch_unwind(|| { let callback = callback.borrow_mut().take(); if let Some(callback) = callback { callback(opened.as_bool()); } })",
  "NSDictionary::<UIApplicationOpenExternalURLOptionsKey, AnyObject>::dictionary()",
  "UIApplication::sharedApplication(main_thread.into_objc2())",
  "application.openURL_options_completionHandler(&url, &options, Some(&handler))",
 )):
  return False
 if compact_body.find("return Err(OpenApplicationUriError::FoundationRejectedUri)") > compact_body.find("Rc::new(RefCell::new(Some(completion)))"):
  return False
 if compact_body.find("return Err(OpenApplicationUriError::MissingScheme)") > compact_body.find("Rc::new(RefCell::new(Some(completion)))"):
  return False
 if compact_body.find("return Err(OpenApplicationUriError::MissingScheme)") > compact_body.find("application.openURL_options_completionHandler"):
  return False
 if any(marker in compact_body for marker in (
  "https",
  "canOpenURL",
  "applicationState",
  ".host(",
  "authority",
  ".replace(",
 )):
  return False
 if not all(fragment in link_probe for fragment in (
  "open_application_uri(",
  '"sample+app://open?item=1"',
  "|_| {},",
 )):
  return False
 if not all(phrase in docs for phrase in (
  "foreground checks",
  "scheme allowlist",
  "sends no uikit request",
  "does not call the completion",
  "does not prove that the target app launched",
  "not available to app extensions",
 )):
  return False
 manifest = json.loads(sources[MANIFEST_PATH])
 rows = {row.get("id"): row for row in manifest.get("capabilities", [])}
 row = rows.get(CAPABILITY_ID, {})
 status = row.get("status_reason", "")
 metadata = row.get("metadata_verification", "")
 if not all(fragment in status for fragment in ("B512", "open_application_uri", "scheme policy remain caller-owned")):
  return False
 if not all(fragment in metadata for fragment in ("B512", "NSURL URLWithString:/scheme", "UIApplication openURL:options:completionHandler:")):
  return False
 spec = json.loads(sources[SPEC_PATH])
 pilot = next((item for item in spec.get("pilots", []) if item.get("id") == "ios-app-lifecycle"), None)
 if pilot is None or not pilot.get("adapters"):
  return False
 adapter = next((item for item in pilot["adapters"] if item.get("id") == ADAPTER_ID), None)
 if adapter is None or not REQUIRED_INPUTS.issubset(set(adapter.get("input_paths", []))):
  return False
 return True


def check_application_uri_negative_regressions(sources):
 mutations = (
  (LIB_PATH, "open_application_uri,", "missing_application_uri,"),
  (PLATFORM_PATH, "url.scheme().is_none_or", "url.host().is_none_or"),
  (PLATFORM_PATH, "FoundationRejectedUri);", "MissingScheme);"),
  (PLATFORM_PATH, "callback.borrow_mut().take()", "callback.borrow_mut().as_ref()"),
  (PLATFORM_PATH, "ios_runtime::ffi::catch_unwind", "ios_runtime::ffi::missing_unwind_guard"),
  (PLATFORM_PATH, "let text = NSString::from_str(uri);", 'if !uri.starts_with("https:") { return Err(OpenApplicationUriError::MissingScheme); } let text = NSString::from_str(uri);'),
  (PLATFORM_PATH, "url.scheme().is_none_or(|scheme| scheme.length() == 0)", "url.scheme().is_none_or(|scheme| scheme.length() == 0) || url.host().is_none_or(|host| host.length() == 0)"),
  (LINK_PROBE_PATH, '"sample+app://open?item=1"', '"relative/path"'),
  (MANIFEST_PATH, "B512 adds `open_application_uri", "B513 adds `open_application_uri"),
 )
 for path, before, after in mutations:
  if before not in sources[path]:
   return False
  mutated = dict(sources)
  mutated[path] = sources[path].replace(before, after, 1)
  if check_application_uri_contract(mutated):
   return False
 mutated = dict(sources)
 spec = json.loads(sources[SPEC_PATH])
 pilot = next(item for item in spec["pilots"] if item.get("id") == "ios-app-lifecycle")
 adapter = next(item for item in pilot["adapters"] if item.get("id") == ADAPTER_ID)
 adapter["input_paths"].remove(PLATFORM_PATH)
 mutated[SPEC_PATH] = json.dumps(spec)
 if check_application_uri_contract(mutated):
  return False
 return True


def check_lifecycle_handler_contract(sources):
 platform = rust_without_comments(sources[PLATFORM_PATH])
 observers = rust_without_comments(sources[OBSERVERS_PATH])
 queue = rust_without_comments(sources[QUEUE_PATH])
 link_probe = rust_without_comments(sources[LINK_PROBE_PATH])
 docs = " ".join(sources[DOC_PATH].split()).lower()
 if not all(re.search(pattern, platform) for pattern in (
  r"pub\s+fn\s+set_event_handler\s*\(\s*&mut\s+self",
  r"pub\s+fn\s+clear_event_handler\s*\(\s*&mut\s+self",
  r"pub\s+fn\s+dropped_handler_event_count\s*\(\s*&self\)\s*->\s*u64",
  r"handler\s*:\s*Arc<LifecycleEventHandlerState>",
  r"impl\s+Drop\s+for\s+IosApplicationLifecycle",
 )):
  return False
 if not re.search(r"pub(?:\([^)]*\))?\s+fn\s+apply\s*\([^)]*\)\s*->\s*Option<LifecycleEvent>", queue):
  return False
 if not re.search(r"HANDLER_PENDING_CAPACITY\s*:\s*usize\s*=\s*16", observers):
  return False
 if not re.search(r"pending\s*:\s*\[Option<LifecycleEvent>;\s*HANDLER_PENDING_CAPACITY\]", observers):
  return False
 if not re.search(r"let event = events\s*\.lock\(\)\s*\.unwrap_or_else\(PoisonError::into_inner\)\s*\.apply\(notification\);\s*if let \(Some\(event\), Some\(handler\)\) = \(event, handler\.upgrade\(\)\)\s*\{\s*handler\.dispatch\(event\);", observers):
  return False
 if not all(fragment in link_probe for fragment in (
  "lifecycle.set_event_handler(|_event| {});",
  "lifecycle.dropped_handler_event_count()",
  "lifecycle.clear_event_handler();",
 )):
  return False
 if not all(name in observers for name in (
  "direct_handler_runs_after_queue_update_and_polling_stays_available",
  "direct_handler_receives_events_after_poll_queue_overflow",
  "reentrant_handler_delivery_is_serialized_and_bounded",
  "clear_rejects_nested_and_later_stale_events",
  "replacement_does_not_restore_the_current_handler_or_deliver_stale_backlog",
  "callback_panic_is_contained_and_handler_remains_installed",
  "callback_does_not_retain_or_update_event_state_after_drop",
 )):
  return False
 return all(fragment in docs for fragment in (
  "synchronously on `nsoperationqueue::mainqueue()`",
  "polling-queue overflow",
  "16-event handler backlog",
  "dropped_handler_event_count()",
  "cannot interrupt a callback already executing",
  "panics are contained",
 ))


def check_lifecycle_handler_negative_regressions(sources):
 mutations = (
  (PLATFORM_PATH, "pub fn set_event_handler", "fn set_event_handler"),
  (PLATFORM_PATH, "pub fn clear_event_handler", "fn clear_event_handler"),
  (OBSERVERS_PATH, "HANDLER_PENDING_CAPACITY: usize = 16", "HANDLER_PENDING_CAPACITY: usize = 8"),
  (OBSERVERS_PATH, "fn clear_rejects_nested_and_later_stale_events", "fn clear_no_longer_rejects_stale_events"),
  (OBSERVERS_PATH, "fn callback_panic_is_contained_and_handler_remains_installed", "fn callback_panic_is_not_contained"),
 )
 for path, before, after in mutations:
  if before not in sources[path]:
   return False
  mutated = dict(sources)
  mutated[path] = sources[path].replace(before, after, 1)
  if check_lifecycle_handler_contract(mutated):
   return False
 return True


def check_contract(root, sources):
 library = sources[LIB_PATH]
 platform = sources[PLATFORM_PATH]
 queue = sources[QUEUE_PATH]
 mapping = sources[MAPPING_PATH]
 observers = sources[OBSERVERS_PATH]
 link_probe = rust_without_comments(sources[LINK_PROBE_PATH])
 package = sources[CARGO_PATH]
 check_script = sources[CHECK_PATH]
 docs = " ".join(sources[DOC_PATH].split())
 docs_lower = docs.lower()
 combined = "\n".join((library, platform, queue, mapping, observers))
 code = rust_without_comments(combined)
 platform_code = rust_without_comments(platform)
 queue_code = rust_without_comments(queue)
 mapping_code = rust_without_comments(mapping)
 observers_code = rust_without_comments(observers)
 failures = []

 if 'name = "ios-app-lifecycle"' not in package:
  failures.append("iOS lifecycle Cargo package name is missing")
 if not all(target in check_script for target in ("aarch64-apple-ios", "aarch64-apple-ios-sim")):
  failures.append("package check does not compile both iOS device and Simulator targets")
 if not check_profile_targets(sources[SPEC_PATH]):
  failures.append("registered iOS lifecycle link checks must use 10.0 device and 14.0 Simulator deployment floors")
 elif not check_profile_floor_regressions(sources[SPEC_PATH]):
  failures.append("registered iOS lifecycle profile must reject each changed device/Simulator floor")
 if not check_dispatch_contract(sources):
  failures.append("opt-in main-thread dispatch API, ownership, panic, and compiled ABI contract is incomplete")
 elif not check_dispatch_negative_regressions(sources):
  failures.append("main-thread dispatch default, optional dependency, Send, or link-feature regressions must fail")
 if not check_application_uri_contract(sources):
  failures.append("generic absolute-URI open API, bounded errors, one-shot panic-contained completion, or caller-policy boundary is incomplete")
 elif not check_application_uri_negative_regressions(sources):
  failures.append("application URI scheme, Foundation, one-shot callback, panic, caller-policy, or generic link-probe regressions must fail")
 if not check_lifecycle_handler_contract(sources):
  failures.append("aggregate lifecycle direct handler, polling parity, bounded reentrancy, stale-clear/drop, or registered link coverage is incomplete")
 elif not check_lifecycle_handler_negative_regressions(sources):
  failures.append("aggregate lifecycle handler API, bounded backlog, clear, or panic regressions must fail")
 if not all(fragment in link_probe for fragment in (
  "MainThread::current()",
  "IosApplicationLifecycle::new(main_thread)",
  "lifecycle.state()",
  "lifecycle.poll_event()",
 )):
  failures.append("link probe must construct and call the lifecycle backend")
 if not re.search(r"\bpub\s+struct\s+IosApplicationLifecycle\b", code):
  failures.append("public IosApplicationLifecycle type is missing")
 if not re.search(r"\bpub\s+fn\s+new\s*\([^)]*\bMainThread\b", code, re.S):
  failures.append("IosApplicationLifecycle::new(MainThread) constructor is missing")
 if not re.search(r"impl\s+ApplicationBackend\s+for\s+IosApplicationLifecycle\b", code):
  failures.append("ApplicationBackend implementation is missing")
 if not re.search(r"\b[A-Za-z_]\w*\s*:\s*(?:(?:ios_runtime::)?MainThread|MainThreadMarker)\b", code):
  failures.append("stored MainThread proof for main-thread affinity is missing")

 for notification, event in NOTIFICATION_MAP.items():
  if not near(platform_code, notification, "LifecycleNotification::" + event):
   failures.append("UIApplication notification is not registered to its lifecycle event: " + notification)
 if "NSNotificationCenter" not in platform_code or not re.search(r"\baddObserver\w*", platform_code):
  failures.append("public Notification Center observer registration is missing")
 if (
  "NSOperationQueue" not in platform_code
  or not near(platform_code, "NSOperationQueue::mainQueue", "add_observer", 2500)
  or not near(platform_code, "addObserverForName", "Some(operation_queue)", 400)
 ):
  failures.append("notification callbacks are not bound to NSOperationQueue.mainQueue")
 if (
  not near(platform_code, "RcBlock::new", "callback()", 350)
  or not near(platform_code, "make_observer_callback", "Arc::downgrade", 300)
  or not near(platform_code, "make_observer_callback", "contain_unwind", 300)
  or not near(platform_code, "fn contain_unwind", "ios_runtime::ffi::catch_unwind", 200)
 ):
  failures.append("native notification callback panic containment is missing")
 if not near(observers_code, "Weak<Mutex<EventQueue>>", "events.upgrade", 400):
  failures.append("observer callback must weakly access live event state")
 if not drop_removes_observer(observers_code) or "removeObserver" not in platform_code:
  failures.append("observer removal from a Drop implementation is missing")
 if not re.search(r"\b[A-Za-z_]\w*observers?\s*:", platform_code, re.I):
  failures.append("notification observer tokens are not stored in a field")

 state_mappings = {
  "DidBecomeActive": "Foreground",
  "WillResignActive": "Inactive",
  "WillEnterForeground": "Inactive",
  "DidEnterBackground": "Background",
 }
 for notification, state in state_mappings.items():
  if not near(queue_code, "LifecycleNotification::" + notification, "ApplicationState::" + state, 450):
   failures.append("lifecycle notification state mapping is missing: " + notification)
 if not near(queue_code, "LifecycleNotification::WillTerminate", "ApplicationState::Terminating", 500):
  failures.append("termination notification does not update state to Terminating")
 if not near(queue_code, "LifecycleNotification::WillTerminate", "TerminationRequested", 500):
  failures.append("termination notification does not emit TerminationRequested")
 if re.search(r"(?:state_changed\s*\(\s*ApplicationState::Terminating|StateChanged\s*\(\s*ApplicationState::Terminating)", queue_code):
  failures.append("termination must not emit StateChanged(Terminating)")
 if not re.search(r"\bconst\s+[A-Za-z_]\w*(?:capacity|limit)\w*\s*:\s*usize\s*=\s*16\b", queue_code, re.I):
  failures.append("bounded lifecycle event queue capacity of 16 is missing")
 if "consecutive_duplicate_state_changes_are_coalesced" not in queue_code:
  failures.append("consecutive duplicate lifecycle state coalescing is missing")

 if not check_unknown_state_error(mapping_code):
  failures.append("unknown UIApplicationState must return ErrorKind::Platform with its raw state code")
 if "does not add a new portable api or require framework-wide initialization" not in docs_lower or "a global service, executor, event loop" not in docs_lower:
  failures.append("documentation must state the portable/API and no-runtime boundary")
 if not all(claim in docs_lower for claim in (
  "aggregate app lifecycle",
  "does not identify which scene changed",
  "does not promise one event per scene",
  "do not launch an app",
  "or prove termination behavior",
  "oldest queued event is dropped",
  "api-availability floor of ios 4.0",
  "effective device deployment minimum is ios 10.0",
  "registered simulator deployment minimum is ios 14.0",
  "raw signed native state value",
 )):
  failures.append("documentation must state aggregate/no-scene/runtime limits and the iOS 4.0 raw-state contract")
 if "Swift source" not in docs:
  failures.append("documentation must state that this package adds no Swift source")

 manifest = json.loads(sources[MANIFEST_PATH])
 rows = {row.get("id"): row for row in manifest.get("capabilities", [])}
 row = rows.get(CAPABILITY_ID, {})
 if row.get("implementation_status") != "partial":
  failures.append("application lifecycle capability row must remain partial")
 if row.get("swift_abi_requirement") is not False:
  failures.append("application lifecycle must not require Swift ABI support")
 for pattern in PRIVATE_OR_DELEGATE_PATTERNS:
  if pattern.search(code):
   failures.append("delegate replacement or private/dynamic API marker found: " + pattern.pattern)
   break
 if re.search(r"\bLifecycleEvent::Terminated\b|\bApplicationState::Terminated\b", code):
  failures.append("completed termination event/state is outside this lifecycle contract")
 if has_swift_source(root):
  failures.append("Swift source is present in the iOS lifecycle package")
 return failures


def main():
 try:
  payload = json.load(sys.stdin)
  root, sources = read_inputs(payload)
  failures = check_contract(root, sources)
  if failures:
   emit("FAIL", "; ".join(failures[:6]))
   return
  emit("PASS", "iOS lifecycle, aggregate handler and polling, opt-in main-thread dispatch, generic application URI, deterministic negative regressions, API availability and deployment floors, registered link probe, and partial capability status are present; URI-open and runtime behavior are not proven")
 except Exception as error:
  emit("ERROR-BLOCKED", str(error)[:256])


if __name__ == "__main__":
 main()
