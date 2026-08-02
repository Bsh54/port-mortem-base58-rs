# Original test suite bridge

This directory lets the **original, unmodified** `mr-tron/base58` Go test files run
against the Rust port, so the port is verified by the real suite — not only by the
translation in `tests/parity.rs`.

## How it works

```
original *_test.go (package base58, byte-identical to upstream)
        │  call base58.Encode / Decode / NewAlphabet / ...
        ▼
gotests/base58.go  (thin cgo shim, package base58)
        │  extern "C" calls
        ▼
b58bridge.dll      (crate b58bridge, C-ABI over the base58 crate)
        ▼
the Rust port
```

- `Cargo.toml` / `src/lib.rs` — a small `cdylib` crate exposing `b58_encode`,
  `b58_decode`, `b58_alphabet_ok`, `b58_free`. This is the only place `unsafe`
  is used; the port crate itself stays `#![forbid(unsafe_code)]`.
- `gotests/base58.go` — a cgo package `base58` whose functions forward to the
  Rust library.
- `gotests/base58_test.go`, `gotests/base58_2_test.go` — copied **verbatim** from
  upstream (same SHA-256 as `tests/original/`). They are not edited.

## Run it

```
pwsh bridge/run-original-tests.ps1
```

Requirements: Rust (GNU toolchain), Go, and a C compiler for cgo (MSYS2
`mingw-w64-x86_64-gcc`). See `original-tests.log` for a recorded pass.

## Scope

The two upstream tests that bind to Go package internals cannot cross the FFI
boundary and are covered elsewhere:

- `TestMulAddBase58WordsLEMatchesBigInt` — tests the internal `mulAddBase58WordsLE`
  helper; mirrored by the `mul_add_matches_u128_reference` unit test in `src/fast.rs`.
- the leading-zero test in `hardening_test.go` — reads the unexported
  `Alphabet.encode` field; mirrored by `leading_zeros_preserved_across_chunk_boundaries`
  in `tests/parity.rs`.

Everything else (known vectors, real BTC addresses, alphabet validation with the
original panic contract, and the fast-vs-trivial equivalence sweep) runs directly
against the Rust port.
