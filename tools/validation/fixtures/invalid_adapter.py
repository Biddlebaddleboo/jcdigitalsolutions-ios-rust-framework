import json


def main():
 print(json.dumps({"schema_version": 2, "status": "PASS", "detail": "invalid schema fixture"}, separators=(",", ":")))


if __name__ == "__main__":
 main()
