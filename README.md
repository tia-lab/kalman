# kalman

Minimal Kalman filters for `f64` time series.

## Scope

This crate currently implements two filters:

- **Local level (1D)**: random walk state with scalar observation noise.
- **Local linear trend (2D)**: `[level, trend]` state with scalar observation noise.

The primary APIs are the `_into` functions, which write into caller-provided output slices to avoid per-call allocations:

- `kalman_local_level_filter_into`
- `kalman_local_linear_trend_filter_into`

Convenience wrappers that allocate and return `Vec`s are also provided:

- `kalman_local_level_filter`
- `kalman_local_linear_trend_filter`

All functions return `MathResult<T> = Result<T, MathError>`.

## Performance

The repository includes Criterion benchmarks for both filters over seeded `Vec<f64>` inputs, writing into pre-allocated output buffers (see `benches/kalman.rs`).

Example results (from `benches/BENCHES.md`, Intel Xeon W-2295, Ubuntu 22.04.5, rustc 1.90.0):

| Benchmark   |      n |               Time (low/med/high) | Med ns/sample |
| ----------- | -----: | --------------------------------: | ------------: |
| local level |    100 | 1.0325 µs / 1.0363 µs / 1.0410 µs |         10.36 |
| local trend |    100 | 1.9037 µs / 1.9123 µs / 1.9263 µs |         19.12 |
| local level |  1,000 | 10.199 µs / 10.220 µs / 10.242 µs |         10.22 |
| local trend |  1,000 | 18.866 µs / 18.911 µs / 18.958 µs |         18.91 |
| local level | 10,000 | 102.30 µs / 102.64 µs / 103.02 µs |         10.26 |
| local trend | 10,000 | 185.49 µs / 185.76 µs / 186.07 µs |         18.58 |

Full details (machine, date, and notes): `benches/BENCHES.md`.

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
kalman = "0.1"
```

### Local level filter

```rust
use kalman::{kalman_local_level_filter_into, MathResult};

fn main() -> MathResult<()> {
    let y = vec![1.0, 1.2, 0.9, 1.1];

    // Observation variance (must be > 0)
    let r = 1.0;
    // Process variance (must be >= 0)
    let q = 0.01;

    let init_mean = 0.0;
    let init_var = 1.0;

    let mut mean = vec![0.0; y.len()];
    let mut var = vec![0.0; y.len()];

    kalman_local_level_filter_into(&y, r, q, init_mean, init_var, &mut mean, &mut var)?;
    Ok(())
}
```

### Local linear trend filter

```rust
use kalman::{kalman_local_linear_trend_filter_into, MathResult};

fn main() -> MathResult<()> {
    let y = vec![10.0, 10.25, 10.5, 10.75];

    let r = 1.0;
    let q_level = 0.01;
    let q_trend = 0.001;

    let init_level = 0.0;
    let init_trend = 0.0;
    let init_var_level = 1.0;
    let init_var_trend = 1.0;

    let mut level = vec![0.0; y.len()];
    let mut trend = vec![0.0; y.len()];
    let mut var_level = vec![0.0; y.len()];
    let mut var_trend = vec![0.0; y.len()];

    kalman_local_linear_trend_filter_into(
        &y,
        r,
        q_level,
        q_trend,
        init_level,
        init_trend,
        init_var_level,
        init_var_trend,
        &mut level,
        &mut trend,
        &mut var_level,
        &mut var_trend,
    )?;
    Ok(())
}
```

## Error handling and input checks

Functions validate:

- `y` must be non-empty and all values finite (`NaN`/`±inf` rejected).
- Variances must be finite; `r > 0`, `q >= 0`, initial variances `>= 0`.
- Output slice lengths must match `y.len()`.

Errors are reported via `MathError` (see `src/errors.rs`).

## Tests

Run unit tests:

```bash
cargo test
```

Test sources: `src/tests/test_kalman.rs`.

## Benchmarks

Benchmarks use Criterion (`criterion = "0.8.1"`).

Run benchmarks:

```bash
cargo bench
```

Bench source: `benches/kalman.rs`.

An example benchmark report is checked in at `benches/BENCHES.md`.
