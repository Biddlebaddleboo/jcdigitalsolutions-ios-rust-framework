import json
from pathlib import Path
import sys


ADAPTER_ID = "call-provider-main-queue-contract"
CAPABILITY_ID = "ios-call-provider"
SOURCE_PATHS = (
 "platform/ios/callkit/ios-call-provider/src/platform.rs",
)
SPEC_PATH = "tools/validation/specs/validation-v1.json"
INPUTS = {*SOURCE_PATHS, SPEC_PATH}
DEFERRED_ACTION_CONTRACTS = (
 (SOURCE_PATHS[0], "Self::with_handlers(main_thread, config, Some(Box::new(handler)), None, None)"),
 (SOURCE_PATHS[0], "pub fn new_with_deferred_actions<L, A>("),
 (SOURCE_PATHS[0], "None,\n            Some(Box::new(lifecycle_handler)),\n            Some(Box::new(action_handler)),"),
 (SOURCE_PATHS[0], "if shared.handler.borrow().is_some() {\n        complete_call_action(shared, action, kind);\n    } else {"),
 (SOURCE_PATHS[0], "impl Drop for PendingNativeAction {"),
 (SOURCE_PATHS[0], "completion_gate.submit()?;"),
 (SOURCE_PATHS[0], ".timeout(&action_id);"),
 (SOURCE_PATHS[0], "state.pending_actions.invalidate();"),
 (SOURCE_PATHS[0], "state.pending_actions.expire_call(pending.call_id)"),
 (SOURCE_PATHS[0], "if !gate.begin_resolution() {\n        drop(initial_state);\n        return;\n    }"),
 (SOURCE_PATHS[0], "if !self.completed && !unsafe { self.native.isComplete() } {\n                self.completed = true;\n                unsafe { self.native.fail() };"),
 (SOURCE_PATHS[0], "resolve_deferred_action(\n                    state.get(main_thread),"),
)


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
  (item.get("path"), item.get("text"))
  for item in pilots[0].get("guards", [])
  if item.get("path") in SOURCE_PATHS and item.get("must_exist") is True
 ]
 if not guards or any(
  path not in SOURCE_PATHS or not isinstance(text, str) or not text
  for path, text in guards
 ):
  raise ValueError("registered CallKit source guards are missing or invalid")
 return guards + list(DEFERRED_ACTION_CONTRACTS)


def contract_holds(sources, guards):
 return all(sources[path].count(text) > 0 for path, text in guards)


def main():
 try:
  sources = inputs(json.load(sys.stdin))
  guards = source_guards(sources[SPEC_PATH])
  if not contract_holds(sources, guards):
   emit("FAIL", "CallKit source does not satisfy every registered required-source guard")
   return
  for index, (path, text) in enumerate(guards, start=1):
   mutated = dict(sources)
   mutated[path] = mutated[path].replace(text, "")
   if mutated[path] == sources[path] or contract_holds(mutated, guards):
    emit("FAIL", f"registered CallKit source-guard mutation {index} was not detected")
    return
  emit("PASS", f"all {len(guards)} registered CallKit source guards hold; each negative mutation was rejected")
 except Exception as error:
  emit("ERROR-BLOCKED", str(error)[:256])


if __name__ == "__main__":
 main()
