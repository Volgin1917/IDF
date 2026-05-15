use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum AiClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {status} — {body}")]
    Api { status: u16, body: String },
    #[error("Rate limited, retry after {retry_after}s")]
    RateLimited { retry_after: u64 },
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f64,
    max_tokens: u32,
    response_format: ResponseFormat,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    total_tokens: u32,
}

pub struct AiClient {
    http: HttpClient,
    api_key: String,
    model: String,
    base_url: String,
}

impl AiClient {
    pub fn from_env() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY")
            .expect("OPENAI_API_KEY must be set");
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4".into());
        Self {
            http: HttpClient::new(),
            api_key,
            model,
            base_url: "https://api.openai.com/v1".into(),
        }
    }

    pub fn new(api_key: String, model: String) -> Self {
        Self {
            http: HttpClient::new(),
            api_key,
            model,
            base_url: "https://api.openai.com/v1".into(),
        }
    }

    pub async fn chat_completion(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: u32,
    ) -> Result<(String, Option<u32>), AiClientError> {
        let body = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".into(),
                    content: system_prompt.into(),
                },
                Message {
                    role: "user".into(),
                    content: user_prompt.into(),
                },
            ],
            temperature: 0.3,
            max_tokens,
            response_format: ResponseFormat { type_: "json_object".into() },
        };

        let resp = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if status == 429 {
            let retry_after = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(30);
            return Err(AiClientError::RateLimited { retry_after });
        }

        if !status.is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            return Err(AiClientError::Api {
                status: status.as_u16(),
                body: body_text,
            });
        }

        let chat: ChatResponse = resp.json().await?;
        let content = chat
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        let total_tokens = chat.usage.map(|u| u.total_tokens);

        info!(
            model = self.model,
            tokens = total_tokens,
            "OpenAI chat completion"
        );

        Ok((content, total_tokens))
    }
}
