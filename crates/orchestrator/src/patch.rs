use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct PatchBlock {
    pub content: String,
}

pub fn parse_begin_end_patch(input: &str) -> Result<PatchBlock> {
    let start = input.find("---BEGIN PATCH---").ok_or_else(|| anyhow!("BEGIN PATCH not found"))?;
    let end = input.find("---END PATCH---").ok_or_else(|| anyhow!("END PATCH not found"))?;
    if end <= start { return Err(anyhow!("Invalid PATCH block ordering")); }
    let content = input[start + "---BEGIN PATCH---".len()..end].trim().to_string();
    Ok(PatchBlock { content })
}
