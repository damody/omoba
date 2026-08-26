from __future__ import annotations

import argparse
import random

from common import load_json, write_result


def main() -> int:
    parser = argparse.ArgumentParser(description="對 encoded-frame manifest 產生 deterministic fault schedule")
    parser.add_argument("manifest")
    parser.add_argument("--seed", type=int, required=True)
    parser.add_argument("--drop-rate", type=float, default=0.0)
    parser.add_argument("--duplicate-rate", type=float, default=0.0)
    parser.add_argument("--output")
    args = parser.parse_args()
    if not 0 <= args.drop_rate <= 1 or not 0 <= args.duplicate_rate <= 1:
        parser.error("rates must be in [0, 1]")
    frames = load_json(args.manifest)["frames"]
    rng = random.Random(args.seed)
    schedule = []
    for frame in frames:
        roll = rng.random()
        action = "drop" if roll < args.drop_rate else "duplicate" if roll < args.drop_rate + args.duplicate_rate else "deliver"
        schedule.append({"team_sequence": frame["team_sequence"], "action": action})
    write_result(args.output, {"seed": args.seed, "schedule": schedule})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
