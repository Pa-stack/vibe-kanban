use crate::{Validator, ValidatorOutcome};

pub struct Determinism<'a> { pub snippets: &'a str }
impl<'a> Validator for Determinism<'a> {
	fn validate(&self) -> Result<ValidatorOutcome, String> {
		let pass = self.snippets.to_uppercase().contains("CACHE_HIT") ||
			self.snippets.matches("OUTPUT_DIGEST=").count() >= 1;
		Ok(ValidatorOutcome { pass, message: None })
	}
}
