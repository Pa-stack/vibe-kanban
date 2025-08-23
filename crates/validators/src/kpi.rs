use anyhow::Result;
use crate::{Validator, ValidatorOutcome};

pub struct Kpi;

#[async_trait::async_trait]
impl Validator for Kpi {
    async fn validate(&self) -> Result<ValidatorOutcome> {
        Ok(ValidatorOutcome { pass: true, message: None })
    }
}
