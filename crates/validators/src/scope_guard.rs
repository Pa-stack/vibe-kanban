use crate::{Validator, ValidatorOutcome};

pub struct ScopeGuard<'a> { pub touched: &'a [String], pub allowlist: Option<&'a [String]> }
impl<'a> Validator for ScopeGuard<'a> {
	fn validate(&self) -> Result<ValidatorOutcome, String> {
		for f in self.touched {
			if f.starts_with('/') || f.contains(':') || f.contains("..") {
				return Ok(ValidatorOutcome { pass: false, message: Some(format!("Invalid path: {}", f)) });
			}
		}
		if let Some(allow) = self.allowlist {
			for f in self.touched {
				if !allow.iter().any(|a| f.starts_with(a)) {
					return Ok(ValidatorOutcome { pass: false, message: Some(format!("{} not in allowlist", f)) });
				}
			}
		}
		Ok(ValidatorOutcome { pass: true, message: Some(format!("SCOPE_GUARD: PASS\nTOUCHED_FILES:\n{}", self.touched.join("\n"))) })
	}
}
