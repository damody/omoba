from __future__ import annotations

import argparse
import json

from common import load_json, write_result


def walk(value, path="$", findings=None):
    findings = findings if findings is not None else []
    forbidden = {"master_seed", "canonical_entity_id", "raw_ecs_id", "other_team_state"}
    if isinstance(value, dict):
        for key, child in value.items():
            if key in forbidden:
                findings.append(f"{path}.{key}")
            walk(child, f"{path}.{key}", findings)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            walk(child, f"{path}[{index}]", findings)
    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description="掃描 decoded V2 JSON 的禁止欄位")
    parser.add_argument("input")
    parser.add_argument("--output")
    args = parser.parse_args()
    findings = walk(load_json(args.input))
    write_result(args.output, {"input": args.input, "findings": findings, "passed": not findings})
    return 0 if not findings else 2


if __name__ == "__main__":
    raise SystemExit(main())
