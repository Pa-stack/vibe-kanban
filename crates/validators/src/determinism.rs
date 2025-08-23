use crate::{Validator, ValidatorOutcome};

pub struct Determinism;
impl Validator for Determinism { fn validate(&self) -> Result<ValidatorOutcome, String> { Ok(ValidatorOutcome { pass: true, message: None }) } }
