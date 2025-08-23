use crate::{Validator, ValidatorOutcome};

pub struct ScopeGuard;
impl Validator for ScopeGuard { fn validate(&self) -> Result<ValidatorOutcome, String> { Ok(ValidatorOutcome { pass: true, message: None }) } }
