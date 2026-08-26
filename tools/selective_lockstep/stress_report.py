from __future__ import annotations

import argparse
import statistics

from common import load_json, sha256_file, write_result


def percentile(values, fraction):
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int((len(ordered) - 1) * fraction))]


def main() -> int:
    parser = argparse.ArgumentParser(description="從 JSONL stress samples 產生 immutable summary")
    parser.add_argument("samples")
    parser.add_argument("--output")
    args = parser.parse_args()
    rows = [load_json_line for load_json_line in (__import__("json").loads(line) for line in open(args.samples, encoding="utf-8"))]
    if not rows:
        parser.error("samples must not be empty")
    metrics = {}
    for key in ("cpu_percent", "memory_bytes", "bytes_per_player_second", "authoritative_tick_us"):
        values = [float(row[key]) for row in rows if key in row]
        if values:
            metrics[key] = {"mean": statistics.fmean(values), "p99": percentile(values, 0.99), "max": max(values)}
    write_result(args.output, {"samples_sha256": sha256_file(args.samples), "sample_count": len(rows), "metrics": metrics})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
