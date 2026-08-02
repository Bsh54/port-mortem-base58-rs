# Benchmark methodology

Goal: honest, reproducible latency — not a cherry-picked throughput headline.

## What is measured

- **Per-operation latency**, timed **in process** around each individual
  `encode` / `decode` call (`std::time::Instant` in Rust, `time.Now()` in Go),
  so process-spawn cost is excluded. We report **p50, p99, and mean**, because a
  mean alone hides tail behaviour and both codecs allocate per call.
- **Throughput** (ops/sec) over the same run, as a secondary number.
- **Cold start**: wall-clock time to spawn the binary and exit on empty input,
  reported as the **minimum of 15 runs** (the floor, least polluted by scheduler
  noise). This is the number that matters for CLI / one-shot use.

## Inputs

Deterministic pseudo-random bytes from an xorshift64 seeded per index, identical
in both implementations (`pseudo_random` / `pseudoRandom`). Sizes 32 and 256
bytes. Decode is benchmarked on the base58 of those same inputs, so both
implementations decode identical strings.

## How to reproduce

```
make bench
# or:
cargo build --release
cd oracle-go && go build -o oracle . && cd ..
python3 bench/run.py --rust target/release/base58 --go oracle-go/oracle
```

Results are written to `bench/results.json`. The raw numbers in this repo were
produced on an Intel Pentium N3530 @ 2.16 GHz (a deliberately modest machine);
absolute values will differ elsewhere, but the Rust-vs-Go **ratio** is the point.

## RSS

Peak resident set is measured externally on an identical workload
(`bench encode 256 500000`), because reading a process's own peak working set
portably would pull in a dependency and this crate ships zero dependencies.

Measured (Windows peak working set, sampled during the run): **Rust 343,672 KB
vs Go 528,096 KB**. The absolute figures include the pre-generated 500k-item
dataset that both implementations build identically, so the delta reflects
per-value overhead and GC headroom (Go) versus compact `Vec`/`String` (Rust),
not codec scratch alone — but it is an apples-to-apples comparison.

Reproduce on Windows (sample while alive, since `PeakWorkingSet64` reads 0 after
exit):

```
$p = Start-Process target/release/base58 -ArgumentList 'bench','encode','256','500000' -PassThru
while (-not $p.HasExited) { $p.Refresh(); $peak = $p.PeakWorkingSet64; Start-Sleep -Milliseconds 50 }
$peak / 1KB
```

On Linux: `/usr/bin/time -v target/release/base58 bench encode 256 500000` and
read `Maximum resident set size` (the `peak_rss_kb` helper in `run.py` does this
automatically when `/usr/bin/time` is present).

## Honesty notes

- **Same version, not an old release.** The Go oracle is built against the exact
  source this port was translated from (vendored at
  `third_party/mr-tron-base58`, wired in via a `replace` in `oracle-go/go.mod`),
  not the older `v1.2.0` tag. Otherwise the port would be "winning" against an
  implementation that predates the chunked fast path — an unfair comparison.
- **Go's per-call p50/p99 are unreliable at these speeds.** Go's `time.Now()` on
  Windows has coarse granularity, so sub-microsecond calls frequently measure as
  `0ns`. Treat Go's per-call percentiles as a lower bound and use **throughput
  (ops/sec)**, which is derived from aggregate wall time and is
  resolution-independent, for the cross-implementation comparison. Rust's
  `Instant` is high-resolution, so its percentiles are meaningful in absolute
  terms. To compare tails fairly, re-run on Linux where Go's monotonic clock is
  nanosecond-grained.
- If the port is slower on a given size, that row is reported as-is. A real p99
  regression stated plainly is worth more than a throughput-only win.
- Both implementations use the same allocation strategy per call (a fresh
  `String` / `[]byte`), so this compares the algorithms, not an arena trick.
