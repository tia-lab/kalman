pub mod errors;
pub mod kalman;

pub use errors::MathError;
pub use kalman::*;
pub type MathResult<T> = Result<T, MathError>;

#[cfg(test)]
mod tests;