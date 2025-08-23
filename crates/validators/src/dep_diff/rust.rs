use crate::{Validator, ValidatorOutcome};

pub struct RustDepDiff;
impl Validator for RustDepDiff { fn validate(&self) -> Result<ValidatorOutcome, String> { Ok(ValidatorOutcome { pass: true, message: None }) } }
