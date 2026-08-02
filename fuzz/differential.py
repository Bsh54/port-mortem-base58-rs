#!/usr/bin/env python3
"""Differential fuzzer: compares the Rust port against the Go original.

For each round it draws a batch of random byte strings (biased toward the
interesting shapes: empty, leading zeros, chunk-size boundaries), then:

  1. encodes the batch with both implementations and compares byte-for-byte;
  2. decodes the resulting base58 with both implementations, compares the
     results, and checks that the round-trip reproduces the original bytes;
  3. decodes a batch of adversarial strings (valid alphabet + invalid chars)
     and checks both implementations agree on accept/reject and on the value.

A divergence is any observable difference in output or in accept/reject
behaviour. The run fails (exit 1) if any divergence is found.
"""

import argparse
import os
import random
import subprocess
import sys
import time

BTC = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
BOUNDARY_LENGTHS = [0, 1, 7, 8, 9, 10, 11, 15, 16, 17, 31, 32, 33, 63, 64, 65, 128, 255]


def run(binary, mode, lines):
    payload = ("\n".join(lines) + "\n").encode("ascii")
    proc = subprocess.run([binary, mode], input=payload, stdout=subprocess.PIPE)
    out = proc.stdout.decode("ascii")
    return out.split("\n")[: len(lines)]


def random_bytes(rng):
    length = rng.choice(BOUNDARY_LENGTHS + [rng.randint(0, 64) for _ in range(4)])
    zeros = rng.randint(0, min(length, 12))
    body = bytes(rng.getrandbits(8) for _ in range(length - zeros))
    return bytes(zeros) + body


def random_base58ish(rng):
    length = rng.randint(1, 48)
    if rng.random() < 0.75:
        return "".join(rng.choice(BTC) for _ in range(length))
    pool = BTC + "0OIl +/=\t"
    return "".join(rng.choice(pool) for _ in range(length))


def is_error(line):
    return line.startswith("!")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rust", required=True)
    ap.add_argument("--go", required=True)
    ap.add_argument("--seconds", type=int, default=60)
    ap.add_argument("--batch", type=int, default=2000)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--log", default=os.path.join(os.path.dirname(__file__), "log.txt"))
    args = ap.parse_args()

    rng = random.Random(args.seed)
    log = open(args.log, "w", encoding="ascii")

    def emit(msg):
        print(msg)
        log.write(msg + "\n")

    emit(f"differential fuzz: rust={args.rust} go={args.go}")
    emit(f"seed={args.seed} batch={args.batch} target_seconds={args.seconds}")

    start = time.time()
    total = 0
    divergences = 0
    rounds = 0

    while time.time() - start < args.seconds:
        rounds += 1
        raw = [random_bytes(rng) for _ in range(args.batch)]
        hex_lines = [b.hex() for b in raw]

        rust_enc = run(args.rust, "encode", hex_lines)
        go_enc = run(args.go, "encode", hex_lines)

        for i, (r, g) in enumerate(zip(rust_enc, go_enc)):
            total += 1
            if r != g:
                divergences += 1
                emit(f"ENCODE DIVERGENCE input={hex_lines[i]} rust={r!r} go={g!r}")

        rust_dec = run(args.rust, "decode", rust_enc)
        go_dec = run(args.go, "decode", go_enc)
        for i, (r, g) in enumerate(zip(rust_dec, go_dec)):
            total += 1
            if r != g:
                divergences += 1
                emit(f"DECODE DIVERGENCE enc={rust_enc[i]!r} rust={r!r} go={g!r}")
            elif not is_error(r) and r != hex_lines[i]:
                divergences += 1
                emit(f"ROUNDTRIP DIVERGENCE input={hex_lines[i]} got={r!r}")

        adversarial = [random_base58ish(rng) for _ in range(args.batch)]
        rust_adv = run(args.rust, "decode", adversarial)
        go_adv = run(args.go, "decode", adversarial)
        for i, (r, g) in enumerate(zip(rust_adv, go_adv)):
            total += 1
            if is_error(r) != is_error(g):
                divergences += 1
                emit(f"ACCEPT/REJECT DIVERGENCE input={adversarial[i]!r} rust={r!r} go={g!r}")
            elif not is_error(r) and r != g:
                divergences += 1
                emit(f"DECODE VALUE DIVERGENCE input={adversarial[i]!r} rust={r!r} go={g!r}")

    elapsed = time.time() - start
    emit(f"---")
    emit(f"rounds={rounds} comparisons={total} divergences={divergences} elapsed={elapsed:.1f}s")
    log.close()
    sys.exit(1 if divergences else 0)


if __name__ == "__main__":
    main()
