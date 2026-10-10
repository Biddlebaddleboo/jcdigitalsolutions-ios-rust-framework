import json
from pathlib import Path
import sys


SPEC_PATH = "tools/validation/specs/validation-v1.json"
LINK_CHECK_PATH = "platform/ios/ios-media-authorization/check-link-imports.sh"
LINK_PROBE_PATH = "platform/ios/ios-media-authorization/examples/media_authorization_link_probe.rs"
EXPECTED_TARGETS = [
 {"minimum_os": "10.0", "sdk": "iphoneos", "triple": "aarch64-apple-ios"},
 {"minimum_os": "14.0", "sdk": "iphonesimulator", "triple": "aarch64-apple-ios-sim"},
]
EXPECTED_IMPORTS = ["AVFoundation.framework", "Foundation.framework", "objc_msgSend"]


REQUIRED = {
 "crates/framework-media-authorization/src/lib.rs": [
  "pub trait MediaAuthorizationRequester",
  "type RequestAuthorizationFuture<'a>",
  "fn request_authorization(",
  "first poll",
  "unpolled future starts no native work",
 ],
 "platform/ios/ios-media-authorization/src/platform.rs": [
  "impl MediaAuthorizationRequester for IosMediaAuthorization",
  "av_media_type(media)",
  "IosMediaAuthorizationFuture::new(media)",
 ],
 "platform/ios/ios-media-authorization/src/request.rs": [
  "fn start(&mut self)",
  "if self.started",
  "AVCaptureDevice::requestAccessForMediaType_completionHandler",
  "complete_from_callback(|| IosMediaAuthorization::authorization_status(media))",
  "impl Drop for IosMediaAuthorizationFuture",
  "self.completion.detach()",
 ],
 "platform/ios/ios-media-authorization/src/status.rs": [
  "CaptureMedia::Camera => Some(AvMediaType::Video)",
  "CaptureMedia::Microphone => Some(AvMediaType::Audio)",
 ],
 "platform/ios/ios-media-authorization/check-surface.sh": [
  "requestAccessForMediaType_completionHandler",
  "capture or audio-session API found in authorization adapter",
 ],
 LINK_CHECK_PATH: [
  "minimum=10.0",
  "minimum=14.0",
  "xcrun vtool -show-build",
  "_objc_msgSend",
  "libSystem.B.dylib libobjc.A.dylib",
  "link probe not executed",
 ],
 LINK_PROBE_PATH: [
  "authorization_status(CaptureMedia::Camera)",
  "Pin::new(&mut request).poll",
 ],
 "docs/ios/media-authorization.md": [
  "NSCameraUsageDescription",
  "NSMicrophoneUsageDescription",
  "Dropping an unpolled future starts",
  "do not create a capture device",
  "public API availability floor remains iOS 7.0",
  "release Rust consumer link emits Mach-O minimum iOS 10.0",
  "for Simulator, as inspected with `vtool`",
 ],
 SPEC_PATH: [],
 "docs/capabilities/capability-status.json": [
  "046-camera-audio-media-camera-capture-session-device",
  "047-camera-audio-media-microphone-audio-engine-i-o",
  "NSCameraUsageDescription",
  "NSMicrophoneUsageDescription",
 ],
}

FORBIDDEN_NATIVE = [
 "AVCaptureSession",
 "AVCaptureDeviceInput",
 "AVCapturePhotoOutput",
 "AVCaptureAudioDataOutput",
 "AVAudioEngine",
 "AVAudioSession",
 "CMSampleBuffer",
 "startRunning(",
]


def link_profile_errors(pilot):
 errors = []
 if pilot.get("targets") != EXPECTED_TARGETS:
  errors.append("compile deployment targets do not match the Rust target floors")
 if pilot.get("abi_targets") != EXPECTED_TARGETS:
  errors.append("ABI deployment targets do not match the linked Rust target floors")
 if pilot.get("abi_scripts") != [LINK_CHECK_PATH]:
  errors.append("device and Simulator link/import probe is not registered")
 if pilot.get("expected_imports") != EXPECTED_IMPORTS:
  errors.append("expected AVFoundation/Foundation/Objective-C imports differ")
 return errors


def rejects_wrong_link_configuration(pilot):
 wrong_floor = dict(pilot)
 wrong_floor["abi_targets"] = [dict(target) for target in EXPECTED_TARGETS]
 wrong_floor["abi_targets"][0]["minimum_os"] = "7.0"
 wrong_imports = dict(pilot)
 wrong_imports["expected_imports"] = ["Foundation.framework", "objc_msgSend"]
 return bool(link_profile_errors(wrong_floor)) and bool(link_profile_errors(wrong_imports))


def emit(status, detail):
 print(json.dumps({"schema_version": 1, "status": status, "detail": detail}, separators=(",", ":")))


def main():
 try:
  payload = json.load(sys.stdin)
  if payload.get("schema_version") != 1 or not isinstance(payload.get("input_paths"), list):
   emit("ERROR-BLOCKED", "invalid adapter input schema")
   return
  root = Path(payload["workspace_root"]).resolve(strict=True)
  sources = {}
  for relative in payload["input_paths"]:
   if not isinstance(relative, str):
    emit("ERROR-BLOCKED", "input path is not a string")
    return
   source = (root / relative).resolve(strict=True)
   if root not in source.parents or not source.is_file():
    emit("ERROR-BLOCKED", "input path escaped workspace root or is not a file")
    return
   sources[relative] = source.read_text(encoding="utf-8")
  missing_paths = [path for path in REQUIRED if path not in sources]
  if missing_paths:
   emit("ERROR-BLOCKED", "required capability source path is missing")
   return
  missing = [f"{path}: {text}" for path, required in REQUIRED.items() for text in required if text not in sources[path]]
  spec = json.loads(sources[SPEC_PATH])
  pilots = [pilot for pilot in spec.get("pilots", []) if pilot.get("id") == "ios-media-authorization"]
  if len(pilots) != 1:
   missing.append("validation spec must contain one ios-media-authorization profile")
  else:
   pilot = pilots[0]
   missing.extend(link_profile_errors(pilot))
   if not rejects_wrong_link_configuration(pilot):
    missing.append("negative regression did not reject wrong deployment floor and missing import")
  manifest = json.loads(sources["docs/capabilities/capability-status.json"])
  rows = {row.get("id"): row for row in manifest.get("capabilities", [])}
  expected_keys = {
   "046-camera-audio-media-camera-capture-session-device": ["NSCameraUsageDescription"],
   "047-camera-audio-media-microphone-audio-engine-i-o": ["NSMicrophoneUsageDescription"],
  }
  partial = all(
   rows.get(identifier, {}).get("implementation_status") == "partial"
   and rows.get(identifier, {}).get("required_info_plist_keys") == keys
   for identifier, keys in expected_keys.items()
  )
  native_sources = "\n".join(sources[path] for path in REQUIRED if path.startswith("platform/ios/") and path.endswith(".rs"))
  forbidden = [item for item in FORBIDDEN_NATIVE if item in native_sources]
  if missing or forbidden or not partial:
   reasons = []
   if missing:
    reasons.append("missing source contract: " + "; ".join(missing[:4]))
   if not partial:
    reasons.append("manifest capability status or purpose key mismatch")
   if forbidden:
    reasons.append("capture API is outside this capability slice: " + ", ".join(forbidden))
   detail = "; ".join(reasons)
   emit("FAIL", detail)
   return
  emit("PASS", "lazy camera/microphone requests and capture-exclusion contract are present")
 except Exception as error:
  emit("ERROR-BLOCKED", str(error)[:256])


if __name__ == "__main__":
 main()
