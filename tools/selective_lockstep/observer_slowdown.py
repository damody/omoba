from __future__ import annotations

import argparse

from common import load_json, write_result


def main() -> int:
    parser = argparse.ArgumentParser(description="建立 observer slowdown/overflow fault schedule")
    parser.add_argument("manifest")
    parser.add_argument("--every", type=int, required=True)
    parser.add_argument("--stall-ticks", type=int, required=True)
    parser.add_argument("--output")
    args = parser.parse_args()
    if args.every < 1 or args.stall_ticks < 1:
        parser.error("--every and --stall-ticks must be positive")
    frames = load_json(args.manifest)["frames"]
    stalls = [{"team_sequence": f["team_sequence"], "stall_ticks": args.stall_ticks} for i, f in enumerate(frames, 1) if i % args.every == 0]
    write_result(args.output, {"stalls": stalls, "outbound_must_continue": True})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
