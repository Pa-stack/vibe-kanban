use anyhow::Result;
use crate::{Validator, ValidatorOutcome};

pub struct ScopeGuard;

#[async_trait::async_trait]
impl Validator for ScopeGuard {
    async fn validate(&self) -> Result<ValidatorOutcome> {
        Ok(ValidatorOutcome { pass: true, message: None })
    }
}
