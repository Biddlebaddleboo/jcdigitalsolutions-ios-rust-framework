import json
from pathlib import Path
import sys


ADAPTER_ID = "call-provider-main-queue-contract"
CAPABILITY_ID = "ios-call-provider"
SOURCE_PATH = "platform/ios/callkit/ios-call-provider/src/platform.rs"
SPEC_PATH = "tools/validation/specs/validation-v1.json"
INPUTS = {SOURCE_PATH, SPEC_PATH}


def emit(status, detail):
 print(json.dumps({"schema_version": 1, "status": status, "detail": detail}, separators=(",", ":")))


def inputs(payload):
 if payload.get("schema_version") != 1 or payload.get("id") != ADAPTER_ID:
  raise ValueError("invalid adapter protocol version or id")
 paths = payload.get("input_paths")
 if not isinstance(paths, list) or set(paths) != INPUTS or len(paths) != len(INPUTS):
  raise ValueError("unexpected CallKit adapter inputs")
 root = Path(payload["workspace_root"]).resolve(strict=True)
 sources = {}
 total = 0
 for relative in paths:
  candidate = Path(relative)
  if candidate.is_absolute() or ".." in candidate.parts:
   raise ValueError("adapter path is not workspace-relative")
  path = (root / candidate).resolve(strict=True)
  if root not in path.parents or not path.is_file():
   raise ValueError("adapter input escaped workspace or is not a file")
  total += path.stat().st_size
  if path.stat().st_size > 1_048_576 or total > 2_097_152:
   raise ValueError("adapter input exceeds the source-size budget")
  sources[relative] = path.read_text(encoding="utf-8")
 return sources


def source_guards(spec_text):
 spec = json.loads(spec_text)
 pilots = [item for item in spec.get("pilots", []) if item.get("id") == CAPABILITY_ID]
 if len(pilots) != 1 or pilots[0].get("package") != CAPABILITY_ID:
  raise ValueError("registered CallKit provider profile is missing or ambiguous")
 guards = [
  item.get("text")
  for item in pilots[0].get("guards", [])
  if item.get("path") == SOURCE_PATH and item.get("must_exist") is True
 ]
 if not guards or any(not isinstance(text, str) or not text for text in guards):
  raise ValueError("registered CallKit source guards are missing or invalid")
 return guards


def contract_holds(source, guards):
 return all(source.count(text) == 1 for text in guards)


def main():
 try:
  sources = inputs(json.load(sys.stdin))
  source = sources[SOURCE_PATH]
  guards = source_guards(sources[SPEC_PATH])
  if not contract_holds(source, guards):
   emit("FAIL", "CallKit source does not satisfy every registered required-source guard")
   return
  for index, text in enumerate(guards, start=1):
   mutated = source.replace(text, "", 1)
   if mutated == source or contract_holds(mutated, guards):
    emit("FAIL", f"registered CallKit source-guard mutation {index} was not detected")
    return
  emit("PASS", f"all {len(guards)} registered CallKit source guards hold; each negative mutation was rejected")
 except Exception as error:
  emit("ERROR-BLOCKED", str(error)[:256])


if __name__ == "__main__":
 main()
