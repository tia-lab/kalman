use criterion::{criterion_group, criterion_main, Criterion};
use kalman::{kalman_local_level_filter_into, kalman_local_linear_trend_filter_into};

use std::hint::black_box;

fn gen_seeded(n: usize, seed: u64) -> Vec<f64> {
    let mut x = Vec::with_capacity(n);
    let mut s = seed;
    for _ in 0..n {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let u = ((s >> 11) as f64) * (1.0 / ((1u64 << 53) as f64));
        x.push(2.0 * u - 1.0);
    }
    x
}

fn bench_kalman(c: &mut Criterion) {
    for &n in &[100usize, 1_000usize, 10_000usize] {
        let y = gen_seeded(n, 123);

        let mut mean = vec![0.0f64; n];
        let mut var = vec![0.0f64; n];
        c.bench_function(&format!("signal_kalman_local_level_ws_n{n}"), |b| {
            b.iter(|| {
                kalman_local_level_filter_into(
                    black_box(&y),
                    1.0,
                    0.01,
                    0.0,
                    1.0,
                    &mut mean,
                    &mut var,
                )
                .unwrap();
                black_box(&mean);
                black_box(&var);
            })
        });

        let mut level = vec![0.0f64; n];
        let mut trend = vec![0.0f64; n];
        let mut var_level = vec![0.0f64; n];
        let mut var_trend = vec![0.0f64; n];
        c.bench_function(&format!("signal_kalman_local_trend_ws_n{n}"), |b| {
            b.iter(|| {
                kalman_local_linear_trend_filter_into(
                    black_box(&y),
                    1.0,
                    0.01,
                    0.001,
                    0.0,
                    0.0,
                    1.0,
                    1.0,
                    &mut level,
                    &mut trend,
                    &mut var_level,
                    &mut var_trend,
                )
                .unwrap();
                black_box(&level);
                black_box(&trend);
                black_box(&var_level);
                black_box(&var_trend);
            })
        });
    }
}

criterion_group!(benches, bench_kalman);
criterion_main!(benches);
