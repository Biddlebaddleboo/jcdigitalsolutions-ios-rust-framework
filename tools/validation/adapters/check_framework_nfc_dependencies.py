import json
from pathlib import Path
import subprocess
import sys


REQUIRED_INPUTS = {
 "Cargo.toml",
 "Cargo.lock",
 ".cargo/config.toml",
 "crates/framework-nfc/Cargo.toml",
}
FORBIDDEN = ("block2", "dispatch2", "objc2", "objc2-core-nfc")
PORTABLE_FIXTURE = "framework-nfc v0.1.0\nframework-core v0.1.0"


def emit(status, detail):
 print(json.dumps({"schema_version": 1, "status": status, "detail": detail}, separators=(",", ":")))


def graph_errors(graph):
 names = {line.split(" ", 1)[0] for line in graph.splitlines() if line.strip()}
 errors = []
 if "framework-core" not in names:
  errors.append("framework-core is missing from the portable dependency graph")
 unexpected = sorted(set(FORBIDDEN) & names)
 if unexpected:
  errors.append("Apple runtime dependencies leaked into framework-nfc: " + ", ".join(unexpected))
 return errors


def main():
 try:
  payload = json.load(sys.stdin)
  if payload.get("schema_version") != 1 or not isinstance(payload.get("input_paths"), list):
   emit("ERROR-BLOCKED", "invalid adapter input schema")
   return
  root = Path(payload["workspace_root"]).resolve(strict=True)
  inputs = set(payload["input_paths"])
  if not REQUIRED_INPUTS.issubset(inputs):
   emit("ERROR-BLOCKED", "Cargo workspace, lockfile, configuration, and framework-nfc manifest must be declared inputs")
   return
  for relative in inputs:
   source = (root / relative).resolve(strict=True)
   if root not in source.parents or not source.is_file():
    emit("ERROR-BLOCKED", "adapter input escaped workspace root or is not a file")
    return
  if not graph_errors("framework-nfc v0.1.0"):
   emit("ERROR-BLOCKED", "missing framework-core negative regression did not fail")
   return
  for dependency in FORBIDDEN:
   if not graph_errors(PORTABLE_FIXTURE + "\n" + dependency + " v0.0.0"):
    emit("ERROR-BLOCKED", "forbidden-dependency negative regression did not fail for " + dependency)
    return
  result = subprocess.run(
   ["cargo", "+1.94.1", "tree", "--locked", "--offline", "-p", "framework-nfc", "--no-default-features", "--edges", "normal", "--prefix", "none"],
   cwd=root,
   capture_output=True,
   text=True,
   timeout=20,
   check=False,
  )
  if result.returncode:
   emit("ERROR-BLOCKED", "cargo tree failed: " + result.stderr.strip()[:512])
   return
  errors = graph_errors(result.stdout)
  if errors:
   emit("FAIL", "; ".join(errors))
   return
  emit("PASS", "no-default-features graph contains framework-core and excludes objc2, objc2-core-nfc, block2, and dispatch2")
 except (OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
  emit("ERROR-BLOCKED", "portable dependency check could not run: " + str(error)[:512])


if __name__ == "__main__":
 main()
