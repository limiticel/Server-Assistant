use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::shared::AppError;

#[derive(Clone)]
pub struct OrchestratorClient {
    base_url: String,
    api_key: Option<String>,
    client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolCall {
    pub arguments: Value,
}

impl OrchestratorClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            api_key,
            client: Client::new(),
        }
    }

    pub async fn list_tools(&self) -> Result<Value, AppError> {
        let mut request = self.client.get(format!("{}/api/tools", self.base_url));
        if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(api_key);
        }
        request
            .send()
            .await
            .map_err(|err| AppError::Upstream(err.to_string()))?
            .json_or_error()
            .await
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, AppError> {
        let mut request = self
            .client
            .post(format!("{}/api/tools/{}/call", self.base_url, name))
            .json(&McpToolCall { arguments });
        if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(api_key);
        }
        request
            .send()
            .await
            .map_err(|err| AppError::Upstream(err.to_string()))?
            .json_or_error()
            .await
    }
}

trait JsonOrError {
    async fn json_or_error(self) -> Result<Value, AppError>;
}

impl JsonOrError for Response {
    async fn json_or_error(self) -> Result<Value, AppError> {
        let status = self.status();
        let body = self
            .text()
            .await
            .map_err(|err| AppError::Upstream(err.to_string()))?;

        if !status.is_success() {
            return Err(AppError::Upstream(format!(
                "orchestrator retornou HTTP {status}: {body}"
            )));
        }

        serde_json::from_str(&body).map_err(|err| {
            AppError::Upstream(format!("resposta JSON invalida do orchestrator: {err}"))
        })
    }
}
