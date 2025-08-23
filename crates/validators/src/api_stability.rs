use crate::{Validator, ValidatorOutcome};

pub struct ApiStability;
impl Validator for ApiStability { fn validate(&self) -> Result<ValidatorOutcome, String> { Ok(ValidatorOutcome { pass: true, message: None }) } }
