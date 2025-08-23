use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PromptRequest {
    pub system: String,
    pub user: String,
}

#[derive(Debug, Clone)]
pub struct PromptResponse {
    pub raw_text: String,
}

pub async fn assemble_prompt(_req: &PromptRequest) -> Result<String> {
    Ok("".to_string())
}
