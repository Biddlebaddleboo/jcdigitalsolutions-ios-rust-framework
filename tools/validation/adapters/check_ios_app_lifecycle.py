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
 if LINK_CHECK_PATH not in pilot.get("shell_scripts", []) or LINK_CHECK_PATH not in pilot.get("scoped_files", []):
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
  emit("PASS", "iOS lifecycle source contract, API availability and deployment floors, registered link probe, and partial capability status are present; runtime behavior is not proven")
 except Exception as error:
  emit("ERROR-BLOCKED", str(error)[:256])


if __name__ == "__main__":
 main()
