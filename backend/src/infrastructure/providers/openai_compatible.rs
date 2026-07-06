use async_stream::stream;
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::{Client, Error, Response};
use serde_json::json;

use crate::{
    domain::ai::{
        AiProvider, ChatRequest, ChatResponse, ChatStream, ChatToolCall, EmbeddingRequest,
        EmbeddingResponse,
    },
    shared::AppError,
};

pub struct OpenAiCompatibleProvider {
    kind: String,
    base_url: String,
    api_key: String,
    client: Client,
}

impl OpenAiCompatibleProvider {
    pub fn new(kind: impl Into<String>, base_url: &str, api_key: &str) -> Self {
        Self {
            kind: kind.into(),
            base_url: base_url.trim_end_matches('/').to_owned(),
            api_key: api_key.to_owned(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl AiProvider for OpenAiCompatibleProvider {
    fn kind(&self) -> &str {
        &self.kind
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, AppError> {
        let url = format!("{}/chat/completions", self.base_url);
        let mut builder = self.client.post(&url).json(&request);

        if !self.api_key.is_empty() {
            builder = builder.bearer_auth(&self.api_key);
        }

        let response = builder
            .send()
            .await
            .map_err(|err| format_network_error(&url, err))?;
        let value: serde_json::Value = response_json(response).await?;

        let content = value["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        let tool_calls = value["choices"][0]["message"]["tool_calls"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| serde_json::from_value::<ChatToolCall>(item.clone()).ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let prompt_tokens = value["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
        let completion_tokens = value["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;

        Ok(ChatResponse {
            content,
            prompt_tokens,
            completion_tokens,
            tool_calls,
        })
    }

    async fn chat_stream(&self, mut request: ChatRequest) -> Result<ChatStream, AppError> {
        request.stream = true;
        let url = format!("{}/chat/completions", self.base_url);
        let mut builder = self.client.post(&url).json(&request);

        if !self.api_key.is_empty() {
            builder = builder.bearer_auth(&self.api_key);
        }

        let response = builder
            .send()
            .await
            .map_err(|err| format_network_error(&url, err))?;
        let response = ensure_success(response).await?;
        let mut upstream = response.bytes_stream();
        let stream = stream! {
            let mut buffer = String::new();

            while let Some(chunk) = upstream.next().await {
                let bytes = match chunk {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        yield Err(AppError::Upstream(err.to_string()));
                        return;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&bytes));

                while let Some(index) = buffer.find("\n\n") {
                    let event = buffer[..index].to_owned();
                    buffer = buffer[index + 2..].to_owned();

                    if let Some(content) = extract_stream_event_content(&event) {
                        yield Ok(content);
                    }
                }
            }

            if let Some(content) = extract_stream_event_content(&buffer) {
                yield Ok(content);
            }
        };

        Ok(Box::pin(stream))
    }

    async fn embeddings(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, AppError> {
        let url = format!("{}/embeddings", self.base_url);
        let mut builder = self
            .client
            .post(&url)
            .json(&json!({ "model": request.model, "input": request.input }));

        if !self.api_key.is_empty() {
            builder = builder.bearer_auth(&self.api_key);
        }

        let response = builder
            .send()
            .await
            .map_err(|err| format_network_error(&url, err))?;
        let value: serde_json::Value = response_json(response).await?;

        let embeddings = value["data"]
            .as_array()
            .map(|items| items.as_slice())
            .unwrap_or(&[])
            .iter()
            .filter_map(|item| serde_json::from_value(item["embedding"].clone()).ok())
            .collect();

        Ok(EmbeddingResponse { embeddings })
    }
}

fn format_network_error(url: &str, err: Error) -> AppError {
    let kind = if err.is_connect() {
        "falha de conexao"
    } else if err.is_timeout() {
        "timeout"
    } else if err.is_request() {
        "request invalido"
    } else {
        "erro de rede"
    };

    AppError::Upstream(format!("{kind} ao chamar {url}: {err}"))
}

async fn ensure_success(response: Response) -> Result<Response, AppError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let body = response.text().await.unwrap_or_default();
    Err(AppError::Upstream(format!(
        "provider retornou HTTP {status}: {body}"
    )))
}

async fn response_json(response: Response) -> Result<serde_json::Value, AppError> {
    ensure_success(response)
        .await?
        .json()
        .await
        .map_err(|err| AppError::Upstream(format!("resposta JSON invalida do provider: {err}")))
}

fn extract_stream_delta(data: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(data).ok()?;
    value["choices"][0]["delta"]["content"]
        .as_str()
        .or_else(|| value["choices"][0]["message"]["content"].as_str())
        .map(ToOwned::to_owned)
}

fn extract_stream_event_content(event: &str) -> Option<String> {
    let content = event
        .lines()
        .filter_map(|line| line.trim().strip_prefix("data:"))
        .map(str::trim_start)
        .filter(|data| *data != "[DONE]")
        .filter_map(extract_stream_delta)
        .collect::<String>();

    (!content.is_empty()).then_some(content)
}
