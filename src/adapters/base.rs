use crate::error::Result;
use crate::models::{AdapterChatCompletion, AdapterChatCompletionChunk, Conversation, Model};
use async_trait::async_trait;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub type AdapterStream = Pin<Box<dyn Stream<Item = Result<AdapterChatCompletionChunk>> + Send>>;

#[async_trait]
pub trait BaseAdapter: Send + Sync {
    fn get_model(&self) -> &Model;

    fn set_api_key(&mut self, api_key: String) -> Result<()>;

    async fn execute(
        &self,
        conversation: &Conversation,
        options: &ExecuteOptions,
    ) -> Result<AdapterChatCompletion>;

    async fn execute_stream(
        &self,
        conversation: &Conversation,
        options: &ExecuteOptions,
    ) -> Result<AdapterStream>;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecuteOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
}

impl ResponseFormat {
    pub fn json() -> Self {
        Self {
            format_type: "json_object".to_string(),
        }
    }

    pub fn text() -> Self {
        Self {
            format_type: "text".to_string(),
        }
    }
}
