use crate::{Validator, ValidatorOutcome};

pub struct NodeDepDiff;
impl Validator for NodeDepDiff { fn validate(&self) -> Result<ValidatorOutcome, String> { Ok(ValidatorOutcome { pass: true, message: None }) } }
