use crate::{Validator, ValidatorOutcome};

pub struct RustDepDiff<'a> { pub before: &'a str, pub after: &'a str }
impl<'a> Validator for RustDepDiff<'a> {
	fn validate(&self) -> Result<ValidatorOutcome, String> {
		let pass = self.before == self.after;
		Ok(ValidatorOutcome { pass, message: Some(if pass {"DEP_DIFF: PASS".into()} else {"DEP_DIFF: FAIL".into()}) })
	}
}
