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
