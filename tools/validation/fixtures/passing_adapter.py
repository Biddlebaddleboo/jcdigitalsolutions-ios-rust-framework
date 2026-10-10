import json
import sys


def main():
 payload = json.load(sys.stdin)
 if payload.get("schema_version") != 1 or payload.get("id") != "adapter-pass-fixture":
  response = {"schema_version": 1, "status": "ERROR-BLOCKED", "detail": "invalid fixture input"}
 else:
  response = {"schema_version": 1, "status": "PASS", "detail": "fixture adapter passed"}
 print(json.dumps(response, separators=(",", ":")))


if __name__ == "__main__":
 main()
