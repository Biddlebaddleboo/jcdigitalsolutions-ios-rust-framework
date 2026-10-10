import json
from pathlib import Path
import sys


REQUIRED = {
 "crates/framework-media-authorization/src/lib.rs": [
  "pub trait MediaAuthorizationRequester",
  "type RequestAuthorizationFuture<'a>",
  "fn request_authorization(",
  "first poll",
  "unpolled future starts no native work",
 ],
 "platform/ios/ios-media-authorization/src/lib.rs": [],
 "platform/ios/ios-media-authorization/src/operation.rs": [],
 "platform/ios/ios-media-authorization/src/platform.rs": [
  "impl MediaAuthorizationRequester for IosMediaAuthorization",
  "av_media_type(media)",
  "IosMediaAuthorizationFuture::new(media)",
 ],
 "platform/ios/ios-media-authorization/src/request.rs": [
  "fn start(&mut self)",
  "if self.started",
  "AVCaptureDevice::requestAccessForMediaType_completionHandler",
  "completion.complete(IosMediaAuthorization::authorization_status(media))",
  "impl Drop for IosMediaAuthorizationFuture",
  "self.completion.detach()",
 ],
 "platform/ios/ios-media-authorization/src/status.rs": [
  "CaptureMedia::Camera => Some(AvMediaType::Video)",
  "CaptureMedia::Microphone => Some(AvMediaType::Audio)",
 ],
 "docs/ios/media-authorization.md": [
  "NSCameraUsageDescription",
  "NSMicrophoneUsageDescription",
  "Dropping an unpolled future starts",
  "do not create a capture device",
 ],
 "docs/capabilities/capability-status.json": [
  "046-camera-audio-media-camera-capture-session-device",
  "047-camera-audio-media-microphone-audio-engine-i-o",
  "NSCameraUsageDescription",
  "NSMicrophoneUsageDescription",
 ],
}

FORBIDDEN_NATIVE = [
 "requestRecordPermission",
 "AVCaptureSession",
 "AVCaptureDeviceInput",
 "AVCapturePhotoOutput",
 "AVCaptureAudioDataOutput",
 "AVAudioEngine",
 "AVAudioSession",
 "AVCaptureDevice::devices",
 "devicesWithMediaType",
 "defaultDeviceWithMediaType",
 "deviceWithUniqueID",
 "startRunning",
 "CMSampleBuffer",
]


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
