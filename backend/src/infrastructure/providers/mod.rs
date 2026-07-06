use std::{collections::HashMap, sync::Arc};

use crate::{config::Settings, domain::ai::AiProvider, shared::AppError};

mod openai_compatible;

pub use openai_compatible::OpenAiCompatibleProvider;

#[derive(Clone, Default)]
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn AiProvider>>,
}

impl ProviderRegistry {
    pub fn from_settings(settings: &Settings) -> Self {
        let mut providers: HashMap<String, Arc<dyn AiProvider>> = HashMap::new();

        if let Some(api_key) = &settings.openai_api_key {
            providers.insert(
                "openai".into(),
                Arc::new(OpenAiCompatibleProvider::new(
                    "openai",
                    "https://api.openai.com/v1",
                    api_key,
                )),
            );
        }
        if let Some(api_key) = &settings.deepseek_api_key {
            providers.insert(
                "deepseek".into(),
                Arc::new(OpenAiCompatibleProvider::new(
                    "deepseek",
                    "https://api.deepseek.com/v1",
                    api_key,
                )),
            );
        }
        if let Some(api_key) = &settings.anthropic_api_key {
            providers.insert(
                "anthropic".into(),
                Arc::new(OpenAiCompatibleProvider::new(
                    "anthropic",
                    "https://api.anthropic.com/v1",
                    api_key,
                )),
            );
        }
        if let Some(api_key) = &settings.gemini_api_key {
            providers.insert(
                "gemini".into(),
                Arc::new(OpenAiCompatibleProvider::new(
                    "gemini",
                    "https://generativelanguage.googleapis.com/v1beta/openai",
                    api_key,
                )),
            );
        }
        providers.insert(
            "ollama".into(),
            Arc::new(OpenAiCompatibleProvider::new(
                "ollama",
                &format!("{}/v1", settings.ollama_base_url),
                "",
            )),
        );

        Self { providers }
    }

    pub fn get(&self, name: &str) -> Result<Arc<dyn AiProvider>, AppError> {
        self.providers
            .get(name)
            .cloned()
            .ok_or_else(|| AppError::Validation(format!("provider '{name}' is not configured")))
    }

    pub fn names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}
