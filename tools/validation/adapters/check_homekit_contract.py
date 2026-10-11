import json
from pathlib import Path
import sys


def emit(status, detail):
 print(json.dumps({"schema_version": 1, "status": status, "detail": detail}, separators=(",", ":")))


def main():
 try:
  payload = json.load(sys.stdin)
  if payload.get("schema_version") != 1 or len(payload.get("input_paths", [])) != 1:
   emit("ERROR-BLOCKED", "invalid adapter input schema")
   return
  root = Path(payload["workspace_root"]).resolve(strict=True)
  source = (root / payload["input_paths"][0]).resolve(strict=True)
  if root not in source.parents:
   emit("ERROR-BLOCKED", "input path escaped workspace root")
   return
  text = source.read_text(encoding="utf-8")
  required = [
   "unsafe { accessory.supportsIdentify() }",
   "objc2::available!(ios = 11.3, ..)",
   "unsafe { self.accessory.identifyWithCompletionHandler(&handler) }",
   "objc2::available!(ios = 8.0, ..)",
   "domain: error.domain().to_string(),",
   "code: error.code() as i64,",
   "completion.complete(result)",
  ]
  missing = [item for item in required if item not in text]
  if missing:
   emit("FAIL", "missing selected HomeKit support or operation contract")
   return
  forbidden = ["HMHomeManager::", "requestAuthorization(", ".identify(", "msg_send!(", "sel!("]
  present = [item for item in forbidden if item in text]
  if present:
   emit("FAIL", "HomeKit manager, authorization, or untyped operation found")
   return
  emit("PASS", "typed HomeKit getter and operation contracts are present")
 except Exception as error:
  emit("ERROR-BLOCKED", str(error)[:256])


if __name__ == "__main__":
 main()
