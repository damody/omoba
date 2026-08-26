from __future__ import annotations

import argparse
from pathlib import Path

from common import write_result


def main() -> int:
    parser = argparse.ArgumentParser(description="掃描 decoded packet bytes 的禁止資訊 token")
    parser.add_argument("capture")
    parser.add_argument("--forbidden", action="append", default=[])
    parser.add_argument("--output")
    args = parser.parse_args()
    payload = Path(args.capture).read_bytes()
    matches = [token for token in args.forbidden if token.encode("utf-8") in payload]
    result = {"capture": args.capture, "bytes": len(payload), "matches": matches, "passed": not matches}
    write_result(args.output, result)
    return 0 if not matches else 2


if __name__ == "__main__":
    raise SystemExit(main())
