use anyhow::Result;
use crate::{Validator, ValidatorOutcome};

pub struct RustDepDiff;

#[async_trait::async_trait]
impl Validator for RustDepDiff {
    async fn validate(&self) -> Result<ValidatorOutcome> {
        Ok(ValidatorOutcome { pass: true, message: None })
    }
}
