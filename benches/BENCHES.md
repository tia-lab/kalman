# Benchmarks (Criterion)

Report generated: 2026-01-12T16:17:32+01:00

## Machine

- CPU: Intel(R) Xeon(R) W-2295 CPU @ 3.00GHz (18C/36T)
- OS: Ubuntu 22.04.5 LTS (Jammy)
- Kernel: Linux 5.15.0-156-generic x86_64

## Toolchain

- rustc: 1.90.0 (1159e78c4 2025-09-14)
- cargo: 1.90.0 (840b83a10 2025-07-30)
- criterion: 0.8.1 (`Cargo.toml` dev-dependency)

## What is measured

Bench suite: `benches/kalman.rs`

Each benchmark runs the filter over a seeded `Vec<f64>` of length `n`, writing results into pre-allocated output buffers:

- `signal_kalman_local_level_ws_n{n}`: `kalman_local_level_filter_into`
- `signal_kalman_local_trend_ws_n{n}`: `kalman_local_linear_trend_filter_into`

Measured `n`: 100, 1_000, 10_000.

## Results

Source log: `benches/benches.log`

Times are Criterion estimates in the format `[low, median, high]`.

| Benchmark   |      n |               Time (low/med/high) | Med ns/sample |
| ----------- | -----: | --------------------------------: | ------------: |
| local level |    100 | 1.0325 µs / 1.0363 µs / 1.0410 µs |         10.36 |
| local trend |    100 | 1.9037 µs / 1.9123 µs / 1.9263 µs |         19.12 |
| local level |  1,000 | 10.199 µs / 10.220 µs / 10.242 µs |         10.22 |
| local trend |  1,000 | 18.866 µs / 18.911 µs / 18.958 µs |         18.91 |
| local level | 10,000 | 102.30 µs / 102.64 µs / 103.02 µs |         10.26 |
| local trend | 10,000 | 185.49 µs / 185.76 µs / 186.07 µs |         18.58 |

## Notes

- The log also shows `cargo test` output with all tests marked `ignored`; the repository separately contains a passing test run log in `src/tests/test_kalman.log`.
