import json
from pathlib import Path
import sys


LIB = "platform/ios/ios-notification-responses/src/lib.rs"
PLATFORM = "platform/ios/ios-notification-responses/src/platform.rs"
OPTIONS = "platform/ios/ios-notification-responses/src/presentation.rs"
VERSION_GATE = "let supports_banner = self.banner_requested()\n            && NSProcessInfo::processInfo().isOperatingSystemAtLeastVersion("
REQUIRED = {
 LIB: [
  ("public options export", "pub use presentation::ForegroundPresentationOptions"),
 ],
 PLATFORM: [
  ("default install suppresses foreground presentation", "Self::install_inner(handler, ForegroundPresentationOptions::new())"),
  ("presentation remains opt-in", "Self::install_inner(handler, presentation)"),
  ("single retained delegate install path", "center.setDelegate(Some(delegate_object));"),
  ("foreground delegate callback", "userNotificationCenter:willPresentNotification:withCompletionHandler:"),
  ("iOS 14 banner boundary", "majorVersion: 14"),
  ("banner availability is short-circuited", VERSION_GATE),
  ("modern banner option guard", "if selection.banner {"),
  ("legacy alert fallback guard", "if selection.legacy_alert {"),
  ("selected presentation reaches completion", "completion.run(presentation);"),
  ("panic fallback completes without presentation", ".unwrap_or_else(|_| UNNotificationPresentationOptions::empty());"),
  ("response callback remains installed", "userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:"),
  ("response handler remains invoked", "handler(response);"),
 ],
 OPTIONS: [
  ("legacy alert only applies before banner support", "legacy_alert: self.banner && !supports_banner"),
  ("sound selection remains independent", "sound: self.sound"),
  ("badge selection remains independent", "badge: self.badge"),
 ],
}


def source_errors(sources):
 errors = [
  f"{path}: {label}"
  for path, requirements in REQUIRED.items()
  for label, text in requirements
  if text not in sources[path]
 ]
 platform = sources[PLATFORM]
 if platform.count("center.setDelegate(Some(delegate_object));") != 1:
  errors.append("delegate must be installed exactly once")
 if platform.count("userNotificationCenter:willPresentNotification:withCompletionHandler:") != 1:
  errors.append("foreground delegate callback must be declared exactly once")
 completion = platform.partition("impl<'a> NativePresentationCompletion")[2].partition(
  "impl Drop for NativePresentationCompletion"
 )[0]
 completion_steps = ("if !self.called {", "self.called = true;", "self.block.call((options,))")
 order = [completion.find(text) for text in completion_steps]
 if not completion or any(completion.count(text) != 1 for text in completion_steps):
  errors.append("presentation completion callback must have one exactly-once guard")
 elif min(order) < 0 or order != sorted(order):
  errors.append("presentation completion must mark itself before its single callback")
 if platform.count("self.run(UNNotificationPresentationOptions::empty());") != 1:
  errors.append("dropped presentation completion must use the empty fallback")
 return errors


def rejects_compilable_mutations(sources):
 mutations = [
  (PLATFORM, "&& NSProcessInfo::processInfo().isOperatingSystemAtLeastVersion(",
   "|| NSProcessInfo::processInfo().isOperatingSystemAtLeastVersion(", "banner short-circuit"),
  (PLATFORM, "if selection.legacy_alert {", "if selection.banner {", "legacy alert fallback"),
  (PLATFORM, "completion.run(presentation);",
   "completion.run(UNNotificationPresentationOptions::empty());", "selected completion options"),
 ]
 errors = []
 for path, original, replacement, label in mutations:
  if sources[path].count(original) != 1:
   errors.append(f"negative fixture source is not unique: {label}")
   continue
  mutant = dict(sources)
  mutant[path] = sources[path].replace(original, replacement, 1)
  if not source_errors(mutant):
   errors.append(f"source contract accepted compilable regression: {label}")
 return errors


def emit(status, detail):
 print(json.dumps({"schema_version": 1, "status": status, "detail": detail}, separators=(",", ":")))


def main():
 try:
  payload = json.load(sys.stdin)
  required_paths = (LIB, PLATFORM, OPTIONS)
  if payload.get("schema_version") != 1 or not isinstance(payload.get("input_paths"), list):
   emit("ERROR-BLOCKED", "invalid adapter input schema")
   return
  root = Path(payload["workspace_root"]).resolve(strict=True)
  sources = {}
  for relative in required_paths:
   if relative not in payload["input_paths"]:
    emit("ERROR-BLOCKED", f"required source input is not declared: {relative}")
    return
   path = (root / relative).resolve(strict=True)
   if root not in path.parents or not path.is_file():
    emit("ERROR-BLOCKED", f"source input escaped workspace or is not a file: {relative}")
    return
   sources[relative] = path.read_text(encoding="utf-8")
  errors = source_errors(sources) + rejects_compilable_mutations(sources)
  if errors:
   emit("FAIL", "; ".join(errors[:4]))
  else:
   emit("PASS", "foreground options, availability, fallback, response, and exactly-once completion contracts hold; negative mutations rejected")
 except Exception as error:
  emit("ERROR-BLOCKED", str(error)[:256])


if __name__ == "__main__":
 main()
