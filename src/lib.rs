//! # Martian Adapters
//!
//! Unified chat-completions interface across many LLM providers.
//!
//! This library provides a consistent API for interacting with various LLM providers
//! (OpenAI, Anthropic, Google Gemini, Cohere, etc.) through a unified interface.
//!
//! ## Quick Start
//!
//! ```no_run
//! use martian_adapters::{AdapterFactory, Conversation, Turn, ConversationRole, ExecuteOptions};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the factory with models.dev data
//!     AdapterFactory::init_from_modelsdev().await?;
//!
//!     // Get a model
//!     let model = AdapterFactory::get_model("openai/openai/gpt-4o-mini").await?;
//!
//!     // Create a conversation
//!     let mut conversation = Conversation::new();
//!     conversation.add_turn(martian_adapters::TurnType::Basic(Turn {
//!         role: ConversationRole::User,
//!         content: "Hello, how are you?".to_string(),
//!     }));
//!
//!     // Get all models supporting vision
//!     let vision_models = AdapterFactory::get_supported_models(
//!         Some(martian_adapters::ModelFilter::new().with_vision(true))
//!     ).await;
//!
//!     println!("Found {} vision-enabled models", vision_models.len());
//!
//!     Ok(())
//! }
//! ```

pub mod adapters;
pub mod config;
pub mod error;
pub mod http;
pub mod models;
pub mod utils;

pub use adapters::{
    AdapterFactory, AdapterStream, BaseAdapter, ExecuteOptions, ModelFilter, ResponseFormat,
};
pub use config::{EnvConfig, ProviderDefaults, VendorMappings};
pub use error::{AdapterError, Result};
pub use http::{ClientCache, HttpClient};
pub use models::{
    AdapterChatCompletion, AdapterChatCompletionChunk, Choice, ChunkChoice, ContentEntry,
    ContentEntryData, ContentTurn, Conversation, ConversationRole, Cost, Delta, FunctionCall,
    ImageUrl, Message, Model, ModelCapabilities, ModelInfo, ModelProperties, ModelsDevResponse,
    Provider, TokenUsage, ToolCall, Turn, TurnType,
};
pub use utils::{
    delete_none_values, encode_image_to_base64, process_image_url_anthropic, EMPTY_CONTENT,
};
