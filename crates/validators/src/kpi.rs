use crate::{Validator, ValidatorOutcome};

pub struct Kpi;
impl Validator for Kpi { fn validate(&self) -> Result<ValidatorOutcome, String> { Ok(ValidatorOutcome { pass: true, message: None }) } }
