use crate::{Validator, ValidatorOutcome};

pub struct Kpi<'a> { pub kpi_json: &'a str }
impl<'a> Validator for Kpi<'a> {
	fn validate(&self) -> Result<ValidatorOutcome, String> {
		let v: serde_json::Value = serde_json::from_str(self.kpi_json).map_err(|e| e.to_string())?;
		let warm = v.get("warm_run_sec").and_then(|n| n.as_f64()).unwrap_or(0.0);
		Ok(ValidatorOutcome { pass: warm <= 600.0, message: Some(format!("KPI: {} (warm={:.3}<=600)", if warm<=600.0 {"PASS"} else {"FAIL"}, warm)) })
	}
}
