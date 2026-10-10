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
  required = ["unsafe { accessory.supportsIdentify() }", "objc2::available!(ios = 11.3, ..)"]
  missing = [item for item in required if item not in text]
  if missing:
   emit("FAIL", "missing selected HomeKit contract guard")
   return
  emit("PASS", "selected HomeKit getter and iOS availability evidence present")
 except Exception as error:
  emit("ERROR-BLOCKED", str(error)[:256])


if __name__ == "__main__":
 main()
