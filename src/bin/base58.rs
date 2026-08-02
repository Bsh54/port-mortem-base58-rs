use std::io::{self, BufRead, Write};
use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("encode") => run_lines(encode_line),
        Some("decode") => run_lines(decode_line),
        Some("bench") => run_bench(&args),
        _ => {
            eprintln!("usage: base58 <encode|decode|bench>");
            eprintln!("  encode: hex in -> base58 out (one item per line)");
            eprintln!("  decode: base58 in -> hex out (errors emitted as '!<message>')");
            eprintln!("  bench <encode|decode> <size> <iters>: in-process latency benchmark");
            ExitCode::from(2)
        }
    }
}

fn run_bench(args: &[String]) -> ExitCode {
    let op = args.get(2).map(String::as_str).unwrap_or("encode");
    let size: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(32);
    let iters: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(200_000);

    let inputs: Vec<Vec<u8>> = (0..iters).map(|i| pseudo_random(i as u64, size)).collect();
    let encoded: Vec<String> = inputs.iter().map(|b| base58::encode(b)).collect();

    let mut latencies = Vec::with_capacity(iters);
    let start = Instant::now();
    match op {
        "decode" => {
            for e in &encoded {
                let t = Instant::now();
                let out = base58::decode(e).unwrap();
                latencies.push(t.elapsed().as_nanos() as u64);
                std::hint::black_box(out);
            }
        }
        _ => {
            for b in &inputs {
                let t = Instant::now();
                let out = base58::encode(b);
                latencies.push(t.elapsed().as_nanos() as u64);
                std::hint::black_box(out);
            }
        }
    }
    let wall = start.elapsed().as_secs_f64();

    latencies.sort_unstable();
    let p = |q: f64| latencies[((latencies.len() as f64 * q) as usize).min(latencies.len() - 1)];
    let mean = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;
    let ops = iters as f64 / wall;

    println!(
        "{{\"impl\":\"rust\",\"op\":\"{op}\",\"size\":{size},\"iters\":{iters},\"p50_ns\":{},\"p99_ns\":{},\"mean_ns\":{mean:.1},\"ops_per_sec\":{ops:.0}}}",
        p(0.50),
        p(0.99)
    );
    ExitCode::SUCCESS
}

fn pseudo_random(seed: u64, len: usize) -> Vec<u8> {
    let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15).wrapping_add(1);
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 24) as u8
        })
        .collect()
}

fn run_lines(f: fn(&str) -> String) -> ExitCode {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let result = f(line.trim());
        if writeln!(out, "{result}").is_err() {
            return ExitCode::FAILURE;
        }
    }
    let _ = out.flush();
    ExitCode::SUCCESS
}

fn encode_line(hex: &str) -> String {
    match decode_hex(hex) {
        Ok(bytes) => base58::encode(&bytes),
        Err(_) => "!invalid hex input".to_string(),
    }
}

fn decode_line(s: &str) -> String {
    match base58::decode(s) {
        Ok(bytes) => encode_hex(&bytes),
        Err(e) => format!("!{e}"),
    }
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ()> {
    if s.len() % 2 != 0 {
        return Err(());
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_val(bytes[i])?;
        let lo = hex_val(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn hex_val(b: u8) -> Result<u8, ()> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(()),
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(char::from_digit((b >> 4) as u32, 16).unwrap());
        s.push(char::from_digit((b & 0x0f) as u32, 16).unwrap());
    }
    s
}
