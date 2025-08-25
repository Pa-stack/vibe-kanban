pub mod scope_guard; pub mod determinism; pub mod kpi; pub mod api_stability; pub mod dep_diff;

#[derive(Clone, Default)]
pub struct ValidatorOutcome { pub pass: bool, pub message: Option<String> }

pub trait Validator { fn validate(&self) -> Result<ValidatorOutcome, String>; }
