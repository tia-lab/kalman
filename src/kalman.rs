// ============================================================================
// MATHILDE PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024 MATHILDE. All Rights Reserved.
//
// This source code contains trade secrets and confidential information owned
// exclusively by MATHILDE, protected under Swiss law:
//
// - URG Art. 2(3), 10(3): Computer program copyright protection
// - URG Art. 24: Reverse engineering/decompilation restricted
// - UWG Art. 5-6: Trade secret and confidential information protection
// - StGB Art. 143bis: Unauthorized data access (criminal)
// - StGB Art. 162: Trade secret violation (criminal)
//
// PROHIBITED: Reproduction, copying, modification, distribution, disclosure,
// reverse engineering, decompilation, or derivative works without prior
// written authorization from MATHILDE.
//
// ACCESS REQUIREMENT: Executed NDA with MATHILDE required. Unauthorized
// access or possession violates Swiss law and international treaties.
//
// ALGORITHMS: Mathematical methods and parameters in this file constitute
// trade secrets independent of copyright protection.
//
// Legal Contact: massimo.nicora@wnlegal.ch
// ============================================================================

use crate::{MathError, MathResult};

const TINY_NEG_CLAMP: f64 = 1e-15;

#[inline]
fn clamp_small_negative(x: f64) -> MathResult<f64> {
    if x.is_finite() && x >= 0.0 {
        Ok(x)
    } else if x.is_finite() && x < 0.0 && x.abs() <= TINY_NEG_CLAMP {
        Ok(0.0)
    } else {
        Err(MathError::NumericalInstability(
            "kalman: covariance became negative".to_string(),
        ))
    }
}

fn validate_variance(name: &'static str, v: f64, allow_zero: bool) -> MathResult<()> {
    if !v.is_finite() {
        return Err(MathError::InvalidParameter {
            parameter: name.to_string(),
            value: v,
            constraint: "must be finite".to_string(),
        });
    }
    if allow_zero {
        if v < 0.0 {
            return Err(MathError::InvalidParameter {
                parameter: name.to_string(),
                value: v,
                constraint: "must be >= 0".to_string(),
            });
        }
    } else if v <= 0.0 {
        return Err(MathError::InvalidParameter {
            parameter: name.to_string(),
            value: v,
            constraint: "must be > 0".to_string(),
        });
    }
    Ok(())
}

fn validate_series(y: &[f64], out_mean: &[f64], out_var: &[f64]) -> MathResult<()> {
    if y.is_empty() {
        return Err(MathError::InsufficientDataAlgo {
            required: 1,
            actual: 0,
        });
    }
    if y.iter().any(|v| !v.is_finite()) {
        return Err(MathError::InvalidData(
            "kalman: all observations must be finite".to_string(),
        ));
    }
    if out_mean.len() != y.len() || out_var.len() != y.len() {
        return Err(MathError::InvalidParameter {
            parameter: "out".to_string(),
            value: 0.0,
            constraint: "out_mean and out_var must match y length".to_string(),
        });
    }
    Ok(())
}

/// 1D local-level Kalman filter:
/// - state: `x_t`
/// - observation: `y_t = x_t + v_t`, `v_t ~ N(0, r)`
/// - transition: `x_t = x_{t-1} + w_t`, `w_t ~ N(0, q)`
///
/// Outputs:
/// - `out_mean[t] = E[x_t | y_0..y_t]`
/// - `out_var[t]  = Var[x_t | y_0..y_t]`
pub fn kalman_local_level_filter_into(
    y: &[f64],
    r: f64,
    q: f64,
    init_mean: f64,
    init_var: f64,
    out_mean: &mut [f64],
    out_var: &mut [f64],
) -> MathResult<()> {
    validate_series(y, out_mean, out_var)?;
    if !init_mean.is_finite() {
        return Err(MathError::InvalidParameter {
            parameter: "init_mean".to_string(),
            value: init_mean,
            constraint: "must be finite".to_string(),
        });
    }
    validate_variance("r", r, false)?;
    validate_variance("q", q, true)?;
    validate_variance("init_var", init_var, true)?;

    let mut x = init_mean;
    let mut p = init_var;

    for (t, &obs) in y.iter().enumerate() {
        // Predict.
        let p_pred = clamp_small_negative(p + q)?;

        // Update.
        let s = p_pred + r;
        if !(s.is_finite() && s > 0.0) {
            return Err(MathError::NumericalInstability(
                "kalman: invalid innovation variance".to_string(),
            ));
        }
        let k = p_pred / s;
        let innov = obs - x;
        x = x + k * innov;
        if !x.is_finite() {
            return Err(MathError::NumericalError {
                reason: "kalman: non-finite updated mean".to_string(),
                operation: Some("kalman_local_level_filter_into".to_string()),
            });
        }

        // Joseph form (1D): P = (1-K)^2 P_pred + K^2 R
        let one_minus_k = 1.0 - k;
        let p_upd = (one_minus_k * one_minus_k) * p_pred + (k * k) * r;
        p = clamp_small_negative(p_upd)?;

        out_mean[t] = x;
        out_var[t] = p;
    }
    Ok(())
}

pub fn kalman_local_level_filter(
    y: &[f64],
    r: f64,
    q: f64,
    init_mean: f64,
    init_var: f64,
) -> MathResult<(Vec<f64>, Vec<f64>)> {
    let mut mean = vec![0.0f64; y.len()];
    let mut var = vec![0.0f64; y.len()];
    kalman_local_level_filter_into(y, r, q, init_mean, init_var, &mut mean, &mut var)?;
    Ok((mean, var))
}

/// 2D local linear trend Kalman filter:
/// - state: `[level_t, trend_t]`
/// - observation: `y_t = level_t + v_t`, `v_t ~ N(0, r)`
/// - transition:
///   - `level_t = level_{t-1} + trend_{t-1} + w1_t`, `w1_t ~ N(0, q_level)`
///   - `trend_t = trend_{t-1} + w2_t`, `w2_t ~ N(0, q_trend)`
///
/// Outputs:
/// - `out_level[t] = E[level_t | y_0..y_t]`
/// - `out_trend[t] = E[trend_t | y_0..y_t]`
/// - `out_var_level[t] = Var(level_t | ...)`
/// - `out_var_trend[t] = Var(trend_t | ...)`
pub fn kalman_local_linear_trend_filter_into(
    y: &[f64],
    r: f64,
    q_level: f64,
    q_trend: f64,
    init_level: f64,
    init_trend: f64,
    init_var_level: f64,
    init_var_trend: f64,
    out_level: &mut [f64],
    out_trend: &mut [f64],
    out_var_level: &mut [f64],
    out_var_trend: &mut [f64],
) -> MathResult<()> {
    if y.is_empty() {
        return Err(MathError::InsufficientDataAlgo {
            required: 1,
            actual: 0,
        });
    }
    if y.iter().any(|v| !v.is_finite()) {
        return Err(MathError::InvalidData(
            "kalman: all observations must be finite".to_string(),
        ));
    }
    let n = y.len();
    if out_level.len() != n
        || out_trend.len() != n
        || out_var_level.len() != n
        || out_var_trend.len() != n
    {
        return Err(MathError::InvalidParameter {
            parameter: "out".to_string(),
            value: 0.0,
            constraint: "all output slices must match y length".to_string(),
        });
    }

    if !init_level.is_finite() || !init_trend.is_finite() {
        return Err(MathError::InvalidParameter {
            parameter: "init_state".to_string(),
            value: 0.0,
            constraint: "init_level and init_trend must be finite".to_string(),
        });
    }

    validate_variance("r", r, false)?;
    validate_variance("q_level", q_level, true)?;
    validate_variance("q_trend", q_trend, true)?;
    validate_variance("init_var_level", init_var_level, true)?;
    validate_variance("init_var_trend", init_var_trend, true)?;

    let mut level = init_level;
    let mut trend = init_trend;

    // Covariance P:
    // [p00 p01]
    // [p10 p11] (p10==p01)
    let mut p00 = init_var_level;
    let mut p01 = 0.0f64;
    let mut p11 = init_var_trend;

    for (t, &obs) in y.iter().enumerate() {
        // Predict:
        // x^- = F x, F=[[1,1],[0,1]]
        let level_pred = level + trend;
        let trend_pred = trend;

        // P^- = F P F^T + Q, with Q = diag(q_level, q_trend).
        // Compute FPF^T explicitly:
        // p00' = p00 + p01 + p10 + p11 = p00 + 2*p01 + p11
        // p01' = p01 + p11
        // p11' = p11
        let p00_pred = clamp_small_negative(p00 + 2.0 * p01 + p11 + q_level)?;
        let p01_pred = p01 + p11;
        let p11_pred = clamp_small_negative(p11 + q_trend)?;
        if !p01_pred.is_finite() {
            return Err(MathError::NumericalError {
                reason: "kalman: non-finite predicted covariance".to_string(),
                operation: Some("kalman_local_linear_trend_filter_into".to_string()),
            });
        }

        // Update with H=[1,0]:
        // S = p00_pred + r
        let s = p00_pred + r;
        if !(s.is_finite() && s > 0.0) {
            return Err(MathError::NumericalInstability(
                "kalman: invalid innovation variance".to_string(),
            ));
        }

        // K = P^- H^T / S = [p00_pred/s, p10_pred/s] where p10_pred==p01_pred
        let k0 = p00_pred / s;
        let k1 = p01_pred / s;
        if !(k0.is_finite() && k1.is_finite()) {
            return Err(MathError::NumericalError {
                reason: "kalman: non-finite gain".to_string(),
                operation: Some("kalman_local_linear_trend_filter_into".to_string()),
            });
        }

        let innov = obs - level_pred;
        if !innov.is_finite() {
            return Err(MathError::NumericalError {
                reason: "kalman: non-finite innovation".to_string(),
                operation: Some("kalman_local_linear_trend_filter_into".to_string()),
            });
        }

        level = level_pred + k0 * innov;
        trend = trend_pred + k1 * innov;
        if !(level.is_finite() && trend.is_finite()) {
            return Err(MathError::NumericalError {
                reason: "kalman: non-finite updated state".to_string(),
                operation: Some("kalman_local_linear_trend_filter_into".to_string()),
            });
        }

        // Joseph form: P = (I-KH)P^-(I-KH)^T + K R K^T
        // For H=[1,0], K=[k0,k1]^T:
        // I-KH = [[1-k0, 0], [-k1, 1]]
        // Compute A = (I-KH)P^-:
        // a00 = (1-k0)*p00_pred
        // a01 = (1-k0)*p01_pred
        // a10 = -k1*p00_pred + p01_pred
        // a11 = -k1*p01_pred + p11_pred
        let one_minus_k0 = 1.0 - k0;
        let a00 = one_minus_k0 * p00_pred;
        let a01 = one_minus_k0 * p01_pred;
        let a10 = -k1 * p00_pred + p01_pred;
        let a11 = -k1 * p01_pred + p11_pred;

        // P_tmp = A (I-KH)^T:
        // p00 = a00*(1-k0) + a01*0 = a00*(1-k0)
        // p01 = a00*(-k1) + a01*1
        // p11 = a10*(-k1) + a11*1
        let mut p00_new = a00 * one_minus_k0;
        let mut p01_new = a00 * (-k1) + a01;
        let mut p11_new = a10 * (-k1) + a11;

        // + K R K^T:
        p00_new += (k0 * k0) * r;
        p01_new += (k0 * k1) * r;
        p11_new += (k1 * k1) * r;

        if !(p00_new.is_finite() && p01_new.is_finite() && p11_new.is_finite()) {
            return Err(MathError::NumericalError {
                reason: "kalman: non-finite updated covariance".to_string(),
                operation: Some("kalman_local_linear_trend_filter_into".to_string()),
            });
        }

        p00 = clamp_small_negative(p00_new)?;
        p11 = clamp_small_negative(p11_new)?;
        p01 = p01_new;

        out_level[t] = level;
        out_trend[t] = trend;
        out_var_level[t] = p00;
        out_var_trend[t] = p11;
    }
    Ok(())
}

pub fn kalman_local_linear_trend_filter(
    y: &[f64],
    r: f64,
    q_level: f64,
    q_trend: f64,
    init_level: f64,
    init_trend: f64,
    init_var_level: f64,
    init_var_trend: f64,
) -> MathResult<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>)> {
    let n = y.len();
    let mut level = vec![0.0f64; n];
    let mut trend = vec![0.0f64; n];
    let mut var_level = vec![0.0f64; n];
    let mut var_trend = vec![0.0f64; n];
    kalman_local_linear_trend_filter_into(
        y,
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
    Ok((level, trend, var_level, var_trend))
}
