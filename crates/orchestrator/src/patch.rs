#[derive(Debug, Clone)]
pub struct PatchBlock {
    pub content: String,
}

pub fn parse_begin_end_patch(input: &str) -> Result<PatchBlock, String> {
    let start = input.find("---BEGIN PATCH---").ok_or_else(|| "BEGIN PATCH not found".to_string())?;
    let end = input.find("---END PATCH---").ok_or_else(|| "END PATCH not found".to_string())?;
    if end <= start { return Err("Invalid PATCH block ordering".to_string()); }
    let content = input[start + "---BEGIN PATCH---".len()..end].trim().to_string();
    Ok(PatchBlock { content })
}
