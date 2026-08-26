from __future__ import annotations

import argparse
import copy

from common import load_json, write_result


def main() -> int:
    parser = argparse.ArgumentParser(description="建立 non-interference paired-world fixture")
    parser.add_argument("input", help="符合 fixture.schema.json 的 JSON")
    parser.add_argument("--output")
    args = parser.parse_args()
    fixture = load_json(args.input)
    shared = fixture["shared"]
    result = {
        "schema_version": fixture["schema_version"],
        "seed": fixture["seed"],
        "ticks": fixture["ticks"],
        "team_under_test": fixture["team_under_test"],
        "world_a": {"shared": copy.deepcopy(shared), "private": fixture["world_a"]},
        "world_b": {"shared": copy.deepcopy(shared), "private": fixture["world_b"]},
        "assertion": "team_under_test projected bytes and hashes must remain identical",
    }
    write_result(args.output, result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
