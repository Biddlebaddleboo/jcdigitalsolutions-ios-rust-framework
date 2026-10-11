import json
import re
import sys
from pathlib import Path


SOURCE_PATH = "platform/ios/ios-camera-capture-session/src/lib.rs"
SELECTOR = "setExposureModeCustomWithDuration_ISO_completionHandler"


def function_body(source, name):
 match = re.search(r"\bpub\s+fn\s+" + re.escape(name) + r"\s*\(", source)
 if match is None:
  return None
 opening = source.find("{", match.end())
 if opening < 0:
  return None
 depth = 0
 for index in range(opening, len(source)):
  if source[index] == "{":
   depth += 1
  elif source[index] == "}":
   depth -= 1
   if depth == 0:
    return source[opening + 1:index]
 return None


def selector_is_scoped(source):
 body = function_body(source, "set_custom_exposure")
 return body is not None and SELECTOR in body and source.count(SELECTOR) == 1


def rejects_misplaced_selector():
 valid = "pub fn set_custom_exposure() { native_selector(); } pub fn configure_exposure() {}"
 removed = valid.replace("native_selector();", "")
 moved = valid.replace("native_selector();", "", 1).replace(
  "pub fn configure_exposure() {}",
  "pub fn configure_exposure() { native_selector(); }",
 )
 return selector_is_scoped(valid.replace("native_selector", SELECTOR)) and not selector_is_scoped(
  removed.replace("native_selector", SELECTOR)
 ) and not selector_is_scoped(moved.replace("native_selector", SELECTOR))


def emit(status, detail):
 print(json.dumps({"schema_version": 1, "status": status, "detail": detail}, separators=(",", ":")))


def main():
 try:
  payload = json.load(sys.stdin)
  if payload.get("schema_version") != 1 or not isinstance(payload.get("input_paths"), list):
   emit("ERROR-BLOCKED", "invalid adapter input schema")
   return
  root = Path(payload["workspace_root"]).resolve(strict=True)
  if SOURCE_PATH not in payload["input_paths"]:
   emit("ERROR-BLOCKED", "custom-exposure source input is missing or outside workspace")
   return
  path = (root / SOURCE_PATH).resolve(strict=True)
  if root not in path.parents or not path.is_file():
   emit("ERROR-BLOCKED", "custom-exposure source escaped workspace root or is not a file")
   return
  source = path.read_text(encoding="utf-8")
  if not selector_is_scoped(source):
   emit("FAIL", "native custom-exposure selector must occur only inside set_custom_exposure")
  elif not rejects_misplaced_selector():
   emit("FAIL", "negative source-scope mutations were not rejected")
  else:
   emit("PASS", "native custom-exposure selector is scoped to set_custom_exposure; remove/move mutations fail")
 except Exception as error:
  emit("ERROR-BLOCKED", str(error)[:256])


if __name__ == "__main__":
 main()
