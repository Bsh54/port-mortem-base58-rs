#!/usr/bin/env python3
"""Benchmark harness: Rust port vs Go original.

Reports per-operation latency (p50 / p99 / mean) and throughput measured
*in process* (so process-spawn cost does not pollute the numbers), plus a
separate cold-start measurement. Results are written to bench/results.json.
"""

import argparse
import json
import os
import subprocess
import time


def bench(binary, op, size, iters):
    out = subprocess.run(
        [binary, "bench", op, str(size), str(iters)],
        stdout=subprocess.PIPE, check=True,
    ).stdout.decode().strip()
    return json.loads(out)


def startup_ns(binary, reps=15):
    best = None
    for _ in range(reps):
        t = time.perf_counter_ns()
        subprocess.run([binary, "encode"], input=b"", stdout=subprocess.DEVNULL, check=True)
        dt = time.perf_counter_ns() - t
        best = dt if best is None else min(best, dt)
    return best


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rust", required=True)
    ap.add_argument("--go", required=True)
    ap.add_argument("--iters", type=int, default=200_000)
    ap.add_argument("--sizes", type=int, nargs="+", default=[32, 256])
    ap.add_argument("--out", default=os.path.join(os.path.dirname(__file__), "results.json"))
    args = ap.parse_args()

    results = {"latency": [], "startup": {}}

    for op in ("encode", "decode"):
        for size in args.sizes:
            r = bench(args.rust, op, size, args.iters)
            g = bench(args.go, op, size, args.iters)
            results["latency"].append({"op": op, "size": size, "rust": r, "go": g})
            print(f"{op:6} size={size:4}  "
                  f"rust p50={r['p50_ns']}ns p99={r['p99_ns']}ns {r['ops_per_sec']:.0f} ops/s | "
                  f"go p50={g['p50_ns']}ns p99={g['p99_ns']}ns {g['ops_per_sec']:.0f} ops/s")

    results["startup"] = {
        "rust_ns": startup_ns(args.rust),
        "go_ns": startup_ns(args.go),
    }
    print(f"startup (min of 15): rust={results['startup']['rust_ns']}ns "
          f"go={results['startup']['go_ns']}ns")

    with open(args.out, "w") as f:
        json.dump(results, f, indent=2)
    print(f"written {args.out}")


if __name__ == "__main__":
    main()
