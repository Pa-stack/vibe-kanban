#[derive(Debug, Clone)]
pub struct PatchBlock {
    pub content: String,
}

pub fn parse_blocks(input: &str) -> Result<Vec<PatchBlock>, String> {
    let mut out = Vec::new();
    let mut rest = input;
    loop {
        let s = match rest.find("---BEGIN PATCH---") { Some(i) => i, None => break };
        rest = &rest[s + "---BEGIN PATCH---".len()..];
        let e = rest.find("---END PATCH---").ok_or_else(|| "END PATCH not found".to_string())?;
        let content = rest[..e].trim().to_string();
        out.push(PatchBlock { content });
        rest = &rest[e + "---END PATCH---".len()..];
    }
    if out.is_empty() { return Err("No PATCH blocks found".to_string()); }
    Ok(out)
}
