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

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum MathError {
    /// Not enough data to compute a requested quantity deterministically.
    #[error("insufficient data: required {required}, actual {actual}")]
    InsufficientDataAlgo { required: usize, actual: usize },

    /// Invalid parameter value or domain violation at the API boundary.
    #[error("invalid parameter `{parameter}`={value}: {constraint}")]
    InvalidParameter {
        parameter: String,
        value: f64,
        constraint: String,
    },

    /// Invalid input data (shape, monotonicity, finiteness, positivity, etc.).
    #[error("invalid data: {0}")]
    InvalidData(String),

    /// A numerical operation failed due to invalid domain/support or non-finite intermediate values.
    #[error(
        "numerical error{op}: {reason}",
        op = match operation {
            Some(op) => format!(" in {op}"),
            None => String::new(),
        }
    )]
    NumericalError {
        reason: String,
        operation: Option<String>,
    },

    /// The computation became numerically unstable (ill-conditioning, singularity, overflow risk).
    #[error("numerical instability: {0}")]
    NumericalInstability(String),

    /// Generic deterministic failure for non-convergence or internal consistency checks.
    #[error("calculation error: {0}")]
    CalculationError(String),
}
