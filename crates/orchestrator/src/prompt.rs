#[derive(Debug, Clone)]
pub struct PromptRequest {
    pub system: String,
    pub user: String,
}

#[derive(Debug, Clone)]
pub struct PromptResponse {
    pub raw_text: String,
}

pub async fn assemble_prompt(_req: &PromptRequest) -> Result<String, String> { Ok(String::new()) }
