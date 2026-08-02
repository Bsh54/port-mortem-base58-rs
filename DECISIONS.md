# DECISIONS

Every non-trivial divergence from the Go original (`mr-tron/base58`) and its rationale.
The goal is behavioral equivalence, not source mimicry: the port reproduces the
observable input/output and accept/reject behaviour of the original while reading as
idiomatic Rust.

## 1. Decoder input type: Go `string` → Rust `impl AsRef<[u8]>`

Go strings hold arbitrary bytes, so the original decodes raw byte `0x80` and reports
`"high-bit set on invalid digit"`. A Rust `&str` is guaranteed UTF-8 and cannot hold a
lone `0x80` (it would encode as `C2 80`). To preserve that exact edge case the public
decoders accept `impl AsRef<[u8]>`, so `&str`, `String`, `&[u8]` and `Vec<u8>` all work
and the high-bit path is reachable and identical to the original.

## 2. Fallible construction: `panic` → `Result`

`NewAlphabet` panics on a bad alphabet. The port returns
`Result<Alphabet, AlphabetError>`. Library code should not panic on caller input; the
three panic cases (wrong length, non-ASCII, duplicate) map to three typed error
variants. The original's panic tests become `is_err()` assertions.

## 3. Error model: `fmt.Errorf` strings → typed `DecodeError` enum

Errors are a `DecodeError` enum implementing `std::error::Error` and `Display`. The
`Display` text keeps the original substrings (`zero length string`,
`invalid base58 digit`, `high-bit set on invalid digit`) so any consumer matching on
message text keeps working, while Rust callers get exhaustive matching.

## 4. 128-bit arithmetic: `math/bits` intrinsics → native `u128`

The fast path needs 128÷64 division (`bits.Div64`) and 64×64→128 multiply
(`bits.Mul64`). Rust expresses both directly with `u128`, which is safe, has no
platform intrinsics, and needs no `unsafe`. The divisor (58^10) is `< 2^64` and the
running remainder is `< divisor`, so the wide dividend never overflows `u128`.

## 5. Zero `unsafe`, proven at compile time

The crate declares `#![forbid(unsafe_code)]`. This is a hard, compiler-enforced
guarantee of zero unsafe blocks — the exact property the event's "Zero Unsafe" bonus
asks for. The original C-adjacent `bits`/pointer-swap tricks are unnecessary here.

## 6. Small-buffer stack optimizations dropped

The original uses fixed stack arrays (`smallWords [8]uint64`, `smallChunks [12]uint64`)
to avoid heap allocation for short inputs. The port uses `Vec` with a pre-reserved
capacity computed from the same size estimate. This trades a micro-optimization for
clarity; for the target input sizes the allocator cost is negligible and the encode
buffer is still sized once, up front (see §9).

## 7. `Trivial` reimplemented without a bignum dependency

The original's `Trivial*` path uses `math/big`. The port keeps an independent second
implementation (byte-wise schoolbook base conversion) so the two code paths cross-check
each other exactly as in the original, but implemented with no external crate. This
preserves the single-binary, zero-dependency property and keeps the differential
`fast == trivial` invariant meaningful.

## 8. Alphabets: package `var` → `LazyLock` statics

`BTCAlphabet`/`FlickrAlphabet` are initialised at import in Go. Rust has no import-time
initialisation, so they are `LazyLock<Alphabet>` statics: one-time, thread-safe, and
constructed through the same validated `Alphabet::new` path.

## 9. Encode buffer sized once

Encoding computes the exact output length (`zcount + ms_digits + full_chunks*10`) and
writes into a single pre-sized buffer from the back, mirroring the original's one
`make([]byte, out_len)`. No incremental `String` growth.

## 10. High-bit check precedes the decode table lookup

The decoder tests `ch > 127` before indexing the 128-entry decode table, exactly
matching the original ordering. This keeps the error classification identical
(`HighBit` vs `InvalidDigit`) for bytes at the ASCII boundary and keeps the table index
in range without bounds tricks.

## 11. Standard-library helpers replace manual bit math

`usize::div_ceil` and `u64::leading_zeros` replace the original's `(x+7)/8` and
`bits.Len64`. Identical results, clearer intent, and no hand-rolled edge cases.

## 12b. Original test suite bridged over FFI, with `unsafe` isolated

To verify the port against the *actual* upstream tests (not only the translation),
the original Go test files run unmodified against the Rust code through a cgo shim
(`bridge/`). The C-ABI glue necessarily uses raw pointers, so it lives in a
**separate `b58bridge` crate** rather than the port: the port keeps
`#![forbid(unsafe_code)]`, and all `unsafe` is confined to test scaffolding. Two
upstream tests bind to Go package internals and cannot cross the FFI boundary; they
are mirrored by a Rust unit test and by `tests/parity.rs`.

## 12. Line-oriented CLI for cheap differential testing

The binary reads one item per line and writes one result per line, so the differential
fuzzer runs a whole batch through a single process instead of spawning per input. This
keeps a 60-second fuzz run practical on modest hardware without changing library
semantics.
