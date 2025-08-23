pub mod scope_guard;
pub mod determinism;
pub mod kpi;
pub mod api_stability;
pub mod dep_diff;

use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct ValidatorOutcome {
    pub pass: bool,
    pub message: Option<String>,
}

#[async_trait::async_trait]
pub trait Validator {
    async fn validate(&self) -> Result<ValidatorOutcome>;
}
