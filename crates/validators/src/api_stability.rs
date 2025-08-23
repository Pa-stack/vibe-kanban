use anyhow::Result;
use crate::{Validator, ValidatorOutcome};

pub struct ApiStability;

#[async_trait::async_trait]
impl Validator for ApiStability {
    async fn validate(&self) -> Result<ValidatorOutcome> {
        Ok(ValidatorOutcome { pass: true, message: None })
    }
}
