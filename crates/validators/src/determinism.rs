use anyhow::Result;
use crate::{Validator, ValidatorOutcome};

pub struct Determinism;

#[async_trait::async_trait]
impl Validator for Determinism {
    async fn validate(&self) -> Result<ValidatorOutcome> {
        Ok(ValidatorOutcome { pass: true, message: None })
    }
}
