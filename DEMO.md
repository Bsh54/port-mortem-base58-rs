# 5-minute demo script

Record a screen capture running these commands top to bottom. Everything is
reproducible from a clean clone; total runtime is a few minutes.

## 0. One-command build (~45s)

```
cargo build --release
```

Point out: no external dependencies, and `#![forbid(unsafe_code)]` at the top of
`src/lib.rs` — the compiler rejects any `unsafe`, so the "Zero Unsafe" claim is a
guarantee, not a promise.

## 1. Parity tests: the translated original suite (~10s)

```
cargo test --release
```

Show the 7 parity tests + the internal `mul_add` unit test passing. Mention that
`tests/original/` holds the unmodified Go suite with SHA-256 hashes, and
`tests/parity.rs` is its 1:1 translation (the Go tests reach into unexported
internals, so they can only run against the Go oracle).

## 2. Known vectors live on the CLI (~10s)

```
printf '61\n626262\n00000000000000000000\n' | ./target/release/base58 encode
printf '2g\na3gV\n1111111111\n' | ./target/release/base58 decode
```

`61 -> 2g`, three zero-preserving `1`s, etc. — matches the BIP/base58 vectors.

## 3. The headline: differential fuzz vs the REAL Go library (~65s)

```
cd oracle-go && go build -o oracle . && cd ..
python3 fuzz/differential.py --rust target/release/base58 --go oracle-go/oracle --seconds 60
```

Let it run. Read the last line out loud:
`comparisons=678000 divergences=0`. This is the behavioral-equivalence proof:
678k random inputs — empty, leading zeros, chunk boundaries — encode, decode,
round-trip, and accept/reject all identical to `mr-tron/base58`.

## 4. Honest numbers (~30s)

```
python3 bench/run.py --rust target/release/base58 --go oracle-go/oracle --iters 50000
```

Show p50 **and** p99, not just throughput. Call out any row where the port is
slower — honesty is the point.

## 5. Close (~20s)

Open `DECISIONS.md`: 12 documented divergences (fallible `Alphabet`, typed
errors, `u128` instead of `math/bits`, byte-input decoder to keep the high-bit
edge case, ...). "The port compiles, passes the original vectors, survives a
differential fuzz against the real library with zero divergences, and every
non-trivial choice is written down."
