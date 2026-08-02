# base58 (Rust) — a behavioral port of `mr-tron/base58`

Port Mortem · Track E (Go → Rust)

A fast, dependency-free, `#![forbid(unsafe_code)]` Base58 codec that reproduces the
observable behaviour of the Go original [`mr-tron/base58`](https://github.com/mr-tron/base58)
— same encodings, same round-trips, same accept/reject decisions, including edge cases
such as leading zeros across chunk boundaries and raw high-bit bytes.

**Results at a glance**

- Builds in one command, zero dependencies, zero `unsafe` (enforced by the compiler).
- Original test suite translated 1:1 — all parity tests pass.
- **630,000 differential comparisons against the real Go binary, 0 divergences.**
- Faster cold start and faster on larger inputs; honest, mixed benchmark below.

## Why this port

The original is the canonical fast Base58 library in Go. Track E is the "is GC
negotiable / predictable behaviour" story. This port keeps the two-implementation
design (an optimized `fast` path and an independent `trivial` path that cross-check each
other) while dropping every escape hatch: no `unsafe`, no external crates, one binary.

The interesting claim is not "it compiles" but "it behaves": see
[`DECISIONS.md`](DECISIONS.md) for every divergence and [`fuzz/`](fuzz) for the
differential proof against the real Go binary.

## Build (one command)

```
cargo build --release
```

Produces the library and the `base58` CLI at `target/release/base58`.
A [`Dockerfile`](Dockerfile) and [`Makefile`](Makefile) are provided for a hermetic
build and for the test / fuzz / bench targets.

## Library API

```rust
let encoded = base58::encode(b"hello world");
let decoded = base58::decode(&encoded).unwrap();
assert_eq!(decoded, b"hello world".to_vec());

// custom alphabet, fallible construction (Go panics; this returns Result)
let alph = base58::Alphabet::new(
    "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz",
).unwrap();
let s = base58::encode_alphabet(&[0, 0, 1, 2, 3], &alph);
```

The full surface mirrors the original: `encode`/`decode`, `*_alphabet`, `fast_*`,
`trivial_*`, plus `BTC_ALPHABET` and `FLICKR_ALPHABET`.

## Tests

```
cargo test --release
```

[`tests/parity.rs`](tests/parity.rs) is a faithful translation of the original Go suite:
known vectors, real BTC addresses, leading-zero preservation across chunk boundaries,
`fast == trivial` on boundary payloads, malformed-input rejection, and alphabet
validation. The unmodified original Go tests are kept under
[`tests/original/`](tests/original) with their kickoff hashes in
[`tests/original/HASHES.txt`](tests/original/HASHES.txt); they run against the Go oracle
(see below) and were translated 1:1 into `tests/parity.rs` because they reach into Go
package internals that a non-Go port cannot link against.

## Differential equivalence vs the Go original

[`oracle-go/`](oracle-go) is a tiny CLI that calls the real `mr-tron/base58`.
[`fuzz/differential.py`](fuzz/differential.py) drives both binaries over random inputs
(biased toward empty, leading-zero, and chunk-boundary shapes), comparing encode output,
decode output, round-trips, and accept/reject decisions.

```
make fuzz          # 60s+ differential run, writes fuzz/log.txt
```

## Benchmarks

Reproducible with `make bench`; methodology and honesty caveats in
[`bench/methodology.md`](bench/methodology.md). The Go oracle is built from the
**same source** this port was translated from (vendored at
`third_party/mr-tron-base58`), not an older release.

Throughput on an Intel Pentium N3530 (higher is better), 50k iterations:

| op     | size | Rust ops/s | Go ops/s | winner |
|--------|------|-----------:|---------:|--------|
| encode | 32   | 426,869    | 673,529  | **Go ~1.6×** |
| encode | 256  | 25,574     | 13,629   | **Rust ~1.9×** |
| decode | 32   | 550,257    | 547,838  | ~parity |
| decode | 256  | 156,007    | 44,051   | **Rust ~3.5×** |

Cold start (min of 15): **Rust ~20 ms vs Go ~36 ms**.

Peak memory (RSS) on an identical `encode 256 × 500k` workload, sampled via the
Windows peak working set: **Rust ~336 MB vs Go ~516 MB** — the port holds ~35 %
less resident memory, the classic no-GC footprint difference.

Honest read: Go's optimized chunked path wins on tiny (32-byte) encodes; the port
pulls ahead as inputs grow, on decode, on startup, and on memory. Go's per-call
p50/p99 read as `0 ns` here because of Windows timer granularity — see the
methodology note; throughput (aggregate wall time) and RSS are the fair
cross-language comparisons on this platform, and a Linux run would additionally
give a trustworthy Go p99.

## License

[MIT](LICENSE), matching the original. The upstream source, used only to build the
differential oracle, is vendored under `third_party/mr-tron-base58` with its own MIT
license preserved.
