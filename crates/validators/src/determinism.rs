use crate::{Validator, ValidatorOutcome};

pub struct Determinism<'a> { pub snippets: &'a str }
impl<'a> Validator for Determinism<'a> {
	fn validate(&self) -> Result<ValidatorOutcome, String> {
		let up = self.snippets.to_uppercase();
		let cache_hit = up.contains("CACHE_HIT");
		let cold = self
			.snippets
			.lines()
			.find(|l| l.starts_with("OUTPUT_DIGEST_COLD="))
			.and_then(|l| l.split('=').nth(1))
			.map(|s| s.trim().to_string());
		let warm = self
			.snippets
			.lines()
			.find(|l| l.starts_with("OUTPUT_DIGEST_WARM="))
			.and_then(|l| l.split('=').nth(1))
			.map(|s| s.trim().to_string());
		let digest_match = matches!((&cold, &warm), (Some(c), Some(w)) if c == w);
		Ok(ValidatorOutcome { pass: cache_hit || digest_match, message: None })
	}
}
