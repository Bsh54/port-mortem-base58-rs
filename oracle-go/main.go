package main

import (
	"bufio"
	"encoding/hex"
	"fmt"
	"os"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/mr-tron/base58"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintln(os.Stderr, "usage: oracle <encode|decode|bench>")
		os.Exit(2)
	}

	if os.Args[1] == "bench" {
		runBench(os.Args)
		return
	}

	in := bufio.NewScanner(os.Stdin)
	in.Buffer(make([]byte, 0, 1024), 16*1024*1024)
	out := bufio.NewWriter(os.Stdout)
	defer out.Flush()

	switch os.Args[1] {
	case "encode":
		for in.Scan() {
			line := strings.TrimSpace(in.Text())
			raw, err := hex.DecodeString(line)
			if err != nil {
				fmt.Fprintln(out, "!invalid hex input")
				continue
			}
			fmt.Fprintln(out, base58.Encode(raw))
		}
	case "decode":
		for in.Scan() {
			line := strings.TrimSpace(in.Text())
			raw, err := base58.Decode(line)
			if err != nil {
				fmt.Fprintln(out, "!"+err.Error())
				continue
			}
			fmt.Fprintln(out, hex.EncodeToString(raw))
		}
	default:
		fmt.Fprintln(os.Stderr, "unknown mode")
		os.Exit(2)
	}
}

func pseudoRandom(seed uint64, n int) []byte {
	state := seed*0x9e3779b97f4a7c15 + 1
	out := make([]byte, n)
	for i := range out {
		state ^= state << 13
		state ^= state >> 7
		state ^= state << 17
		out[i] = byte(state >> 24)
	}
	return out
}

func runBench(args []string) {
	op := "encode"
	if len(args) > 2 {
		op = args[2]
	}
	size := 32
	if len(args) > 3 {
		if v, err := strconv.Atoi(args[3]); err == nil {
			size = v
		}
	}
	iters := 200000
	if len(args) > 4 {
		if v, err := strconv.Atoi(args[4]); err == nil {
			iters = v
		}
	}

	inputs := make([][]byte, iters)
	encoded := make([]string, iters)
	for i := 0; i < iters; i++ {
		inputs[i] = pseudoRandom(uint64(i), size)
		encoded[i] = base58.Encode(inputs[i])
	}

	latencies := make([]int64, iters)
	start := time.Now()
	if op == "decode" {
		for i, e := range encoded {
			t := time.Now()
			out, _ := base58.Decode(e)
			latencies[i] = time.Since(t).Nanoseconds()
			_ = out
		}
	} else {
		for i, b := range inputs {
			t := time.Now()
			out := base58.Encode(b)
			latencies[i] = time.Since(t).Nanoseconds()
			_ = out
		}
	}
	wall := time.Since(start).Seconds()

	sort.Slice(latencies, func(i, j int) bool { return latencies[i] < latencies[j] })
	pct := func(q float64) int64 {
		idx := int(float64(len(latencies)) * q)
		if idx >= len(latencies) {
			idx = len(latencies) - 1
		}
		return latencies[idx]
	}
	var sum int64
	for _, v := range latencies {
		sum += v
	}
	mean := float64(sum) / float64(len(latencies))
	ops := float64(iters) / wall

	fmt.Printf("{\"impl\":\"go\",\"op\":\"%s\",\"size\":%d,\"iters\":%d,\"p50_ns\":%d,\"p99_ns\":%d,\"mean_ns\":%.1f,\"ops_per_sec\":%.0f}\n",
		op, size, iters, pct(0.50), pct(0.99), mean, ops)
}
