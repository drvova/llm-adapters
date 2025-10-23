# Rust Port Design: Leveraging models.dev API

## Executive Summary

This document outlines the design for porting the `martian-adapters` Python package to Rust, with a key architectural shift: leveraging the [models.dev API](https://models.dev/api.json) as the single source of truth for model metadata instead of hardcoding model definitions in each provider adapter. This approach reduces maintenance burden, ensures up-to-date model information, and simplifies the addition of new models and providers.

## 1. Python Architecture Analysis

### 1.1 Core Features & Abstractions

The Python implementation (`martian-adapters`) provides the following core features that the Rust port must replicate:

#### Conversation Types
- **Turn**: Basic message with role (user/assistant/system) and text content
- **ContentTurn**: Multi-modal content with text and image entries
- **FunctionOutputTurn**: Function execution results
- **ToolOutputTurn**: Tool execution results with tool_call_id
- **ToolsCallTurn**: Assistant message containing tool calls
- **FunctionCallTurn**: Assistant message containing function call (deprecated)
- **Conversation**: Collection of turns representing message history

#### Adapter Interface
```
BaseAdapter (abstract)
  ↓
SDKChatAdapter (generic base with CLIENT_SYNC, CLIENT_ASYNC type params)
  ↓
OpenAISDKChatAdapter (OpenAI SDK specific)
  ↓
Provider-specific adapters (OpenAISDKChatProviderAdapter, etc.)
```

Key adapter responsibilities:
- Execute sync/async chat completions
- Handle streaming responses
- Transform conversations to provider-specific formats
- Calculate costs based on token usage
- Manage API keys and HTTP clients
- Validate model capabilities against request parameters

#### Capability Handling

The `Model` class contains ~25 boolean flags describing what each model supports:

| Capability Flag | Description |
|----------------|-------------|
| `supports_streaming` | Server-sent events streaming |
| `supports_vision` | Image input in content |
| `supports_tools` | Tool/function calling |
| `supports_temperature` | Temperature parameter |
| `supports_n` | Multiple completions (n>1) |
| `supports_json_output` | JSON response format |
| `supports_json_content` | JSON structure in messages |
| `supports_system` | System role messages |
| `supports_multiple_system` | Multiple system messages |
| `supports_repeating_roles` | Consecutive same-role messages |
| `supports_user` | User parameter for tracking |
| `supports_empty_content` | Empty string content |
| `supports_tool_choice` | Tool choice parameter |
| `supports_tool_choice_required` | "required" tool choice |
| `supports_last_assistant` | Last message can be assistant |
| `supports_first_assistant` | First message can be assistant |
| `supports_only_system` | Conversation with only system |
| `supports_only_assistant` | Conversation with only assistant |

The adapter validates requests against these capabilities and transforms conversations to work around limitations (e.g., converting system messages to user messages if `supports_system=False`).

#### Streaming Support

- **Sync streaming**: Python generators yielding `AdapterChatCompletionChunk`
- **Async streaming**: AsyncGenerators with automatic cleanup
- Stream state tracking for cost calculation on completion

#### Cost Accounting

- **Cost model**: `Cost { prompt: float, completion: float, request: float }`
- **Per-token pricing**: Multiplied by token counts from usage field
- **Dynamic costs**: Some models (Gemini) have tiered pricing based on context length
- **Reasoning tokens**: Special handling for o1/reasoning models (doubled cost)

#### HTTP Client Layer

- **Client caching**: Cached by `(base_url, api_key, client_type)`
- **Connection pooling**: Configurable via `MAX_CONNECTIONS_PER_PROCESS`, `MAX_KEEPALIVE_CONNECTIONS_PER_PROCESS`
- **Timeouts**: `HTTP_TIMEOUT` (600s default), `HTTP_CONNECT_TIMEOUT` (5s default)
- **Base URL override**: `_ADAPTERS_OVERRIDE_ALL_BASE_URLS_` for testing

### 1.2 Key Components

```
adapters/
├── types.py                    # Pydantic models (Model, Cost, Conversation, Turn types)
├── adapter_factory.py          # Registry mapping model paths to adapter classes
├── constants.py                # Environment-based configuration
├── client_cache.py             # HTTP client caching
├── general_utils.py            # Message normalization, image preprocessing
├── abstract_adapters/
│   ├── base_adapter.py         # BaseAdapter abstract class
│   ├── sdk_chat_adapter.py     # Generic SDK chat adapter base
│   ├── openai_sdk_chat_adapter.py  # OpenAI-compatible SDK base
│   ├── api_key_adapter_mixin.py    # API key management
│   └── provider_adapter_mixin.py   # get_supported_models() interface
└── provider_adapters/          # 20+ provider implementations
    ├── openai_sdk_chat_provider_adapter.py
    ├── anthropic_sdk_chat_provider_adapter.py
    ├── gemini_sdk_chat_provider_adapter.py
    ├── cohere_sdk_chat_provider_adapter.py
    └── ...
```

### 1.3 AdapterFactory Pattern

The factory uses Python introspection to discover adapters:
1. Scans `adapters.provider_adapters` module for classes
2. Filters for classes implementing both `ProviderAdapterMixin` and `BaseAdapter`
3. Calls `get_supported_models()` on each adapter
4. Builds registries: `model_path → adapter_class` and `model_path → Model`

Model paths follow the format: `{provider}/{vendor}/{model_name}` (e.g., `openai/openai/gpt-4o`)

### 1.4 Provider-Specific Behaviors

Each provider adapter encodes provider-specific logic:

- **Temperature adjustment**: Cohere requires `temperature ∈ [0, 2]` (Python default is `[0, 1]`)
- **Message transformations**: Different formats for system messages, tool calls
- **Client creation**: Different SDK initialization (OpenAI vs Anthropic vs Gemini)
- **Response extraction**: Different response schemas
- **Special parameters**: Azure requires `api_version`, Vertex requires `project_id`
- **Image preprocessing**: Anthropic requires base64-encoded images with media_type
- **Tool calling**: Different schemas (OpenAI tools vs Anthropic tools vs Cohere tools)

## 2. models.dev API Structure

### 2.1 API Endpoint

**GET** `https://models.dev/api.json`

Returns a JSON object mapping provider IDs to provider metadata and model lists.

### 2.2 Response Schema

```json
{
  "provider_id": {
    "id": "provider_id",
    "name": "Provider Display Name",
    "env": ["API_KEY_ENV_VAR"],
    "npm": "@ai-sdk/provider-name",
    "api": "https://api.provider.com/v1",  // optional base URL
    "doc": "https://docs.provider.com",
    "models": {
      "model-id": {
        "id": "model-id",
        "name": "Display Name",
        "attachment": true,          // supports file attachments
        "reasoning": false,          // reasoning model (o1, etc.)
        "temperature": true,
        "tool_call": true,
        "knowledge": "2024-10",      // training data cutoff
        "release_date": "2024-10-01",
        "last_updated": "2024-10-01",
        "modalities": {
          "input": ["text", "image", "video"],
          "output": ["text"]
        },
        "open_weights": false,
        "cost": {
          "input": 2.5,              // per million tokens
          "output": 10.0,
          "cache_read": 0.25,        // optional
          "cache_write": 3.75        // optional
        },
        "limit": {
          "context": 128000,
          "output": 16384
        }
      }
    }
  }
}
```

### 2.3 Provider Logo Assets

**GET** `https://models.dev/logos/{provider}.svg`

Returns SVG logo for the provider (useful for UI integrations).

## 3. Field Mapping: Python Model ↔ models.dev

| Python `Model` Field | models.dev Mapping | Transformation |
|---------------------|-------------------|----------------|
| `name` | `model.id` | Direct |
| `vendor_name` | *(inferred)* | Extract from model name or separate mapping table |
| `provider_name` | `provider.id` | Direct |
| `cost.prompt` | `model.cost.input` | Divide by 1,000,000 (convert per-million to per-token) |
| `cost.completion` | `model.cost.output` | Divide by 1,000,000 |
| `cost.request` | *(not present)* | Default to 0.0 |
| `context_length` | `model.limit.context` | Direct |
| `completion_length` | `model.limit.output` | Direct |
| `supports_streaming` | *(not present)* | Default to true (most models support streaming) |
| `supports_vision` | `"image" in model.modalities.input` | Check array membership |
| `supports_tools` | `model.tool_call` | Direct |
| `supports_temperature` | `model.temperature` | Direct |
| `supports_n` | *(not present)* | Default to true |
| `supports_system` | *(not present)* | Provider-specific default |
| `supports_multiple_system` | *(not present)* | Provider-specific default |
| `supports_repeating_roles` | *(not present)* | Provider-specific default |
| `supports_user` | *(not present)* | Default to true |
| `supports_empty_content` | *(not present)* | Provider-specific default |
| `supports_tool_choice` | *(inferred from tool_call)* | Same as `supports_tools` |
| `supports_tool_choice_required` | *(not present)* | Provider-specific default |
| `supports_json_output` | *(not present)* | Provider-specific default |
| `supports_json_content` | *(not present)* | Default to true |
| `supports_last_assistant` | *(not present)* | Default to true |
| `supports_first_assistant` | *(not present)* | Default to true |
| `supports_only_system` | *(not present)* | Default to true |
| `supports_only_assistant` | *(not present)* | Default to true |
| `properties.open_source` | `model.open_weights` | Direct |
| `properties.gdpr_compliant` | *(not present)* | Provider-specific (e.g., OpenAI/Azure true) |
| `properties.chinese` | *(inferred)* | Check provider ID or model name |
| `properties.is_nsfw` | *(not present)* | Default to false |

### 3.1 Gaps & Required Transformations

**Missing in models.dev:**
1. **Granular capability flags**: Most conversation constraint flags (repeating roles, empty content, system message variations)
2. **Vendor information**: No explicit vendor field; must extract from model naming patterns or maintain a mapping
3. **Request cost**: Only token costs provided
4. **Provider-specific quirks**: Temperature ranges, parameter support, message format requirements

**Required transformations:**
1. **Cost conversion**: `cost_per_token = modelsdev_cost / 1_000_000.0`
2. **Vendor extraction**: Pattern matching or lookup table (e.g., `gpt-*` → `openai`, `claude-*` → `anthropic`, `llama*` → `meta-llama`)
3. **Capability defaults**: Per-provider default capability sets based on provider characteristics
4. **Base URL resolution**: Use `provider.api` if present, else use provider-specific hardcoded URLs
5. **Cache cost handling**: Map `cache_read` and `cache_write` to prompt multipliers for certain models

**Reconciliation strategy:**
- **Hybrid approach**: Fetch model metadata from models.dev, but maintain provider-specific configuration files in the Rust crate for:
  - Default capability flags per provider
  - Base URL overrides
  - Special parameter handling
  - Vendor name mappings
  - Provider-specific message transformations

## 4. Rust Crate Design

### 4.1 Crate Structure

```
martian-adapters-rust/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                  # Public API surface
│   ├── models/                 # Data structures
│   │   ├── mod.rs
│   │   ├── conversation.rs     # Turn, Conversation types
│   │   ├── model.rs            # Model, ModelCapabilities structs
│   │   ├── cost.rs             # Cost accounting
│   │   ├── response.rs         # AdapterChatCompletion, streaming types
│   │   └── modelsdev.rs        # models.dev API response types
│   ├── adapters/               # Adapter trait & implementations
│   │   ├── mod.rs
│   │   ├── base.rs             # BaseAdapter trait
│   │   ├── factory.rs          # AdapterFactory
│   │   ├── openai_compat.rs    # OpenAI-compatible base adapter
│   │   └── providers/
│   │       ├── mod.rs
│   │       ├── openai.rs
│   │       ├── anthropic.rs
│   │       ├── gemini.rs
│   │       ├── cohere.rs
│   │       └── ...
│   ├── http/                   # HTTP client layer
│   │   ├── mod.rs
│   │   ├── client.rs           # Reqwest-based HTTP client
│   │   └── cache.rs            # Client caching with DashMap
│   ├── config/                 # Configuration management
│   │   ├── mod.rs
│   │   ├── env.rs              # Environment variables
│   │   ├── provider_defaults.rs # Provider capability defaults
│   │   └── vendor_mappings.rs  # Model name → vendor mappings
│   ├── utils/                  # Utilities
│   │   ├── mod.rs
│   │   ├── normalization.rs    # Message normalization
│   │   └── images.rs           # Image URL preprocessing
│   └── error.rs                # Error types
├── config/                     # Configuration files
│   ├── provider_defaults.toml  # Per-provider capability defaults
│   └── vendor_mappings.toml    # Model name patterns to vendor
└── tests/
    └── integration/
```

### 4.2 Key Dependencies (Cargo.toml)

```toml
[dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Async runtime
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# HTTP client
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Concurrency & caching
dashmap = "5.5"
once_cell = "1.19"

# Configuration
dotenvy = "0.15"  # .env file support
toml = "0.8"

# Image processing
base64 = "0.21"

# Utilities
url = "2.5"
uuid = { version = "1.0", features = ["v4"] }

[dev-dependencies]
tokio-test = "0.4"
mockito = "1.0"
```

### 4.3 Core Data Models

#### 4.3.1 Conversation Types (src/models/conversation.rs)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConversationRole {
    User,
    Assistant,
    System,
    Function,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub role: ConversationRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEntry {
    #[serde(rename = "type")]
    pub entry_type: String,  // "text" or "image_url"
    #[serde(flatten)]
    pub data: ContentEntryData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentEntryData {
    Text { text: String },
    Image { image_url: ImageUrl },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,  // "high", "low", "auto"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTurn {
    pub role: ConversationRole,
    pub content: Vec<ContentEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TurnType {
    Basic(Turn),
    Content(ContentTurn),
    ToolOutput {
        role: ConversationRole,
        content: Option<String>,
        tool_call_id: String,
    },
    ToolCalls {
        role: ConversationRole,
        content: Option<String>,
        tool_calls: Vec<ToolCall>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub turns: Vec<TurnType>,
}
```

#### 4.3.2 Model & Capabilities (src/models/model.rs)

```rust
use serde::{Deserialize, Serialize};
use crate::models::cost::Cost;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub supports_user: bool,
    pub supports_repeating_roles: bool,
    pub supports_streaming: bool,
    pub supports_vision: bool,
    pub supports_tools: bool,
    pub supports_n: bool,
    pub supports_system: bool,
    pub supports_multiple_system: bool,
    pub supports_empty_content: bool,
    pub supports_tool_choice: bool,
    pub supports_tool_choice_required: bool,
    pub supports_json_output: bool,
    pub supports_json_content: bool,
    pub supports_last_assistant: bool,
    pub supports_first_assistant: bool,
    pub supports_temperature: bool,
    pub supports_only_system: bool,
    pub supports_only_assistant: bool,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            supports_user: true,
            supports_repeating_roles: true,
            supports_streaming: true,
            supports_vision: false,
            supports_tools: false,
            supports_n: true,
            supports_system: true,
            supports_multiple_system: true,
            supports_empty_content: true,
            supports_tool_choice: false,
            supports_tool_choice_required: false,
            supports_json_output: true,
            supports_json_content: true,
            supports_last_assistant: true,
            supports_first_assistant: true,
            supports_temperature: true,
            supports_only_system: true,
            supports_only_assistant: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProperties {
    pub open_source: bool,
    pub chinese: bool,
    pub gdpr_compliant: bool,
    pub is_nsfw: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub vendor_name: String,
    pub provider_name: String,
    pub cost: Cost,
    pub context_length: u32,
    pub completion_length: Option<u32>,
    pub capabilities: ModelCapabilities,
    pub properties: ModelProperties,
    
    // Metadata from models.dev
    pub knowledge_cutoff: Option<String>,
    pub release_date: Option<String>,
    pub last_updated: Option<String>,
}

impl Model {
    pub fn get_path(&self) -> String {
        format!("{}/{}/{}", self.provider_name, self.vendor_name, self.name)
    }
}
```

#### 4.3.3 models.dev API Types (src/models/modelsdev.rs)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevResponse {
    #[serde(flatten)]
    pub providers: HashMap<String, Provider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub env: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    pub models: HashMap<String, ModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub attachment: bool,
    #[serde(default)]
    pub reasoning: bool,
    #[serde(default)]
    pub temperature: bool,
    #[serde(default)]
    pub tool_call: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub knowledge: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
    pub modalities: Modalities,
    #[serde(default)]
    pub open_weights: bool,
    pub cost: ModelCost,
    pub limit: ModelLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modalities {
    pub input: Vec<String>,
    pub output: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCost {
    pub input: f64,   // per million tokens
    pub output: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLimit {
    pub context: u32,
    pub output: u32,
}
```

#### 4.3.4 Cost (src/models/cost.rs)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cost {
    pub prompt: f64,      // cost per token
    pub completion: f64,  // cost per token
    pub request: f64,     // flat cost per request
}

impl Cost {
    pub fn from_modelsdev(input_per_million: f64, output_per_million: f64) -> Self {
        Self {
            prompt: input_per_million / 1_000_000.0,
            completion: output_per_million / 1_000_000.0,
            request: 0.0,
        }
    }
    
    pub fn calculate(&self, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        self.prompt * prompt_tokens as f64
            + self.completion * completion_tokens as f64
            + self.request
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

#### 4.3.5 Response Types (src/models/response.rs)

```rust
use serde::{Deserialize, Serialize};
use crate::models::{ConversationRole, TokenUsage, ToolCall};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterChatCompletion {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<TokenUsage>,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: ConversationRole,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkChoice {
    pub index: u32,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<ConversationRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}
```

### 4.4 Adapter Interface

#### 4.4.1 BaseAdapter Trait (src/adapters/base.rs)

```rust
use async_trait::async_trait;
use futures::stream::Stream;
use std::pin::Pin;
use crate::models::{
    Conversation, 
    AdapterChatCompletion, 
    AdapterChatCompletionChunk,
    Model
};
use crate::error::AdapterError;

pub type AdapterStream = Pin<Box<dyn Stream<Item = Result<AdapterChatCompletionChunk, AdapterError>> + Send>>;

#[async_trait]
pub trait BaseAdapter: Send + Sync {
    /// Get the model this adapter is configured for
    fn get_model(&self) -> &Model;
    
    /// Set the API key (for runtime key rotation)
    fn set_api_key(&mut self, api_key: String) -> Result<(), AdapterError>;
    
    /// Execute a chat completion (non-streaming)
    async fn execute(
        &self,
        conversation: &Conversation,
        options: &ExecuteOptions,
    ) -> Result<AdapterChatCompletion, AdapterError>;
    
    /// Execute a chat completion (streaming)
    async fn execute_stream(
        &self,
        conversation: &Conversation,
        options: &ExecuteOptions,
    ) -> Result<AdapterStream, AdapterError>;
}

#[derive(Debug, Clone, Default)]
pub struct ExecuteOptions {
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
    pub tools: Option<Vec<serde_json::Value>>,
    pub tool_choice: Option<String>,
    pub response_format: Option<ResponseFormat>,
    pub n: Option<u32>,
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,  // "text" or "json_object"
}
```

#### 4.4.2 AdapterFactory (src/adapters/factory.rs)

```rust
use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use crate::adapters::BaseAdapter;
use crate::models::{Model, modelsdev::ModelsDevResponse};
use crate::error::AdapterError;
use crate::config::VendorMappings;

pub struct AdapterFactory {
    models: HashMap<String, Model>,
    provider_adapters: HashMap<String, Box<dyn Fn(&Model) -> Box<dyn BaseAdapter>>>,
    vendor_mappings: Arc<VendorMappings>,
}

static FACTORY: Lazy<RwLock<AdapterFactory>> = Lazy::new(|| {
    RwLock::new(AdapterFactory::new())
});

impl AdapterFactory {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            provider_adapters: HashMap::new(),
            vendor_mappings: Arc::new(VendorMappings::load()),
        }
    }
    
    /// Initialize from models.dev API
    pub async fn init_from_modelsdev() -> Result<(), AdapterError> {
        let response = Self::fetch_modelsdev_api().await?;
        let mut factory = FACTORY.write().await;
        factory.populate_from_modelsdev(response)?;
        Ok(())
    }
    
    async fn fetch_modelsdev_api() -> Result<ModelsDevResponse, AdapterError> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://models.dev/api.json")
            .send()
            .await?
            .json::<ModelsDevResponse>()
            .await?;
        Ok(response)
    }
    
    fn populate_from_modelsdev(&mut self, response: ModelsDevResponse) -> Result<(), AdapterError> {
        for (provider_id, provider) in response.providers {
            for (model_id, model_info) in provider.models {
                let model = self.convert_modelsdev_model(
                    &provider_id,
                    &model_id,
                    &model_info,
                )?;
                
                let path = model.get_path();
                self.models.insert(path, model);
            }
        }
        Ok(())
    }
    
    fn convert_modelsdev_model(
        &self,
        provider_id: &str,
        model_id: &str,
        model_info: &crate::models::modelsdev::ModelInfo,
    ) -> Result<Model, AdapterError> {
        use crate::models::{Model, Cost, ModelCapabilities, ModelProperties};
        use crate::config::ProviderDefaults;
        
        // Extract vendor from model name using mappings
        let vendor_name = self.vendor_mappings.extract_vendor(model_id, provider_id);
        
        // Load provider-specific capability defaults
        let defaults = ProviderDefaults::for_provider(provider_id);
        
        // Build capabilities from models.dev + defaults
        let mut capabilities = defaults.capabilities.clone();
        capabilities.supports_vision = model_info.modalities.input.contains(&"image".to_string());
        capabilities.supports_tools = model_info.tool_call;
        capabilities.supports_temperature = model_info.temperature;
        
        Ok(Model {
            name: model_id.to_string(),
            vendor_name,
            provider_name: provider_id.to_string(),
            cost: Cost::from_modelsdev(model_info.cost.input, model_info.cost.output),
            context_length: model_info.limit.context,
            completion_length: Some(model_info.limit.output),
            capabilities,
            properties: ModelProperties {
                open_source: model_info.open_weights,
                chinese: self.is_chinese_model(model_id, provider_id),
                gdpr_compliant: self.is_gdpr_compliant(provider_id),
                is_nsfw: false,
            },
            knowledge_cutoff: model_info.knowledge.clone(),
            release_date: model_info.release_date.clone(),
            last_updated: model_info.last_updated.clone(),
        })
    }
    
    /// Register a provider adapter constructor
    pub fn register_provider<F>(&mut self, provider_id: &str, constructor: F)
    where
        F: Fn(&Model) -> Box<dyn BaseAdapter> + 'static,
    {
        self.provider_adapters.insert(provider_id.to_string(), Box::new(constructor));
    }
    
    /// Get an adapter for a specific model path
    pub async fn get_adapter(model_path: &str) -> Result<Box<dyn BaseAdapter>, AdapterError> {
        let factory = FACTORY.read().await;
        
        let model = factory.models
            .get(model_path)
            .ok_or_else(|| AdapterError::ModelNotFound(model_path.to_string()))?;
        
        let constructor = factory.provider_adapters
            .get(&model.provider_name)
            .ok_or_else(|| AdapterError::ProviderNotSupported(model.provider_name.clone()))?;
        
        Ok(constructor(model))
    }
    
    /// Get model by path
    pub async fn get_model(model_path: &str) -> Result<Model, AdapterError> {
        let factory = FACTORY.read().await;
        factory.models
            .get(model_path)
            .cloned()
            .ok_or_else(|| AdapterError::ModelNotFound(model_path.to_string()))
    }
    
    /// Get all supported models with optional filtering
    pub async fn get_supported_models(filter: Option<ModelFilter>) -> Vec<Model> {
        let factory = FACTORY.read().await;
        factory.models
            .values()
            .filter(|model| {
                if let Some(ref f) = filter {
                    f.matches(model)
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }
    
    fn is_chinese_model(&self, model_id: &str, provider_id: &str) -> bool {
        provider_id.contains("china") 
            || provider_id.contains("alibaba")
            || provider_id.contains("moonshot")
            || model_id.contains("qwen")
    }
    
    fn is_gdpr_compliant(&self, provider_id: &str) -> bool {
        matches!(provider_id, "openai" | "azure" | "anthropic")
    }
}

#[derive(Debug, Clone, Default)]
pub struct ModelFilter {
    pub supports_streaming: Option<bool>,
    pub supports_vision: Option<bool>,
    pub supports_tools: Option<bool>,
    pub supports_temperature: Option<bool>,
    pub provider: Option<String>,
}

impl ModelFilter {
    fn matches(&self, model: &Model) -> bool {
        if let Some(streaming) = self.supports_streaming {
            if model.capabilities.supports_streaming != streaming {
                return false;
            }
        }
        if let Some(vision) = self.supports_vision {
            if model.capabilities.supports_vision != vision {
                return false;
            }
        }
        if let Some(tools) = self.supports_tools {
            if model.capabilities.supports_tools != tools {
                return false;
            }
        }
        if let Some(temp) = self.supports_temperature {
            if model.capabilities.supports_temperature != temp {
                return false;
            }
        }
        if let Some(ref prov) = self.provider {
            if &model.provider_name != prov {
                return false;
            }
        }
        true
    }
}
```

### 4.5 HTTP Client Layer

#### 4.5.1 Client with Caching (src/http/client.rs)

```rust
use reqwest::{Client, ClientBuilder};
use std::time::Duration;
use crate::error::AdapterError;

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Result<Self, AdapterError> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(600))
            .connect_timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(100)
            .build()?;
        
        Ok(Self { client })
    }
    
    pub fn with_timeout(timeout_secs: u64) -> Result<Self, AdapterError> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout_secs))
            .connect_timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(100)
            .build()?;
        
        Ok(Self { client })
    }
    
    pub fn inner(&self) -> &Client {
        &self.client
    }
}
```

#### 4.5.2 Client Cache (src/http/cache.rs)

```rust
use dashmap::DashMap;
use once_cell::sync::Lazy;
use crate::http::HttpClient;

type CacheKey = (String, String);  // (base_url, api_key_hash)

static CLIENT_CACHE: Lazy<DashMap<CacheKey, HttpClient>> = Lazy::new(DashMap::new);

pub struct ClientCache;

impl ClientCache {
    pub fn get_or_create(base_url: &str, api_key: &str) -> HttpClient {
        let key = Self::make_key(base_url, api_key);
        
        CLIENT_CACHE
            .entry(key)
            .or_insert_with(|| HttpClient::new().expect("Failed to create HTTP client"))
            .clone()
    }
    
    fn make_key(base_url: &str, api_key: &str) -> CacheKey {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        api_key.hash(&mut hasher);
        let api_key_hash = format!("{:x}", hasher.finish());
        
        (base_url.to_string(), api_key_hash)
    }
    
    pub fn clear() {
        CLIENT_CACHE.clear();
    }
}
```

### 4.6 Configuration Management

#### 4.6.1 Provider Defaults (config/provider_defaults.toml)

```toml
[openai]
supports_system = true
supports_multiple_system = true
supports_repeating_roles = true
supports_empty_content = true
supports_tool_choice = true
supports_tool_choice_required = true
supports_json_output = true
supports_json_content = true

[anthropic]
supports_system = true
supports_multiple_system = false  # Only first system message
supports_repeating_roles = false
supports_empty_content = false
supports_tool_choice = true
supports_tool_choice_required = true
supports_json_output = true
supports_json_content = true

[cohere]
supports_system = true
supports_multiple_system = false
supports_repeating_roles = false
supports_empty_content = false
supports_tool_choice = false
supports_json_output = true
supports_json_content = false

# ... more providers
```

#### 4.6.2 Vendor Mappings (config/vendor_mappings.toml)

```toml
# Pattern-based vendor extraction
[patterns]
"^gpt-" = "openai"
"^claude-" = "anthropic"
"^llama" = "meta-llama"
"^gemini-" = "gemini"
"^command-" = "cohere"
"^mixtral-" = "mistralai"
"^qwen" = "qwen"
"^mistral-" = "mistralai"

# Provider-specific overrides
[provider_defaults]
openai = "openai"
anthropic = "anthropic"
cohere = "cohere"
gemini = "gemini"
```

#### 4.6.3 Config Loader (src/config/provider_defaults.rs)

```rust
use serde::Deserialize;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use crate::models::ModelCapabilities;

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderDefaults {
    pub capabilities: ModelCapabilities,
    #[serde(default)]
    pub base_url: Option<String>,
}

static PROVIDER_DEFAULTS: Lazy<HashMap<String, ProviderDefaults>> = Lazy::new(|| {
    let config_str = include_str!("../../config/provider_defaults.toml");
    toml::from_str(config_str).expect("Failed to parse provider_defaults.toml")
});

impl ProviderDefaults {
    pub fn for_provider(provider_id: &str) -> &'static ProviderDefaults {
        static DEFAULT: Lazy<ProviderDefaults> = Lazy::new(|| ProviderDefaults {
            capabilities: ModelCapabilities::default(),
            base_url: None,
        });
        
        PROVIDER_DEFAULTS.get(provider_id).unwrap_or(&DEFAULT)
    }
}
```

### 4.7 Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Provider not supported: {0}")]
    ProviderNotSupported(String),
    
    #[error("API key not found for provider: {0}")]
    ApiKeyNotFound(String),
    
    #[error("Model does not support {feature}: {model}")]
    UnsupportedFeature { model: String, feature: String },
    
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Stream error: {0}")]
    StreamError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}
```

## 5. Provider-Specific Behaviors

### 5.1 Reconciliation Strategy

The Rust port will use a **hybrid approach** for provider-specific behaviors:

1. **models.dev for model metadata**: Cost, context length, basic capabilities
2. **Provider config files for quirks**: Capability defaults, message transformation rules
3. **Provider adapter implementations for SDK differences**: Client initialization, request/response transformation

### 5.2 Provider Adapter Example: OpenAI

```rust
// src/adapters/providers/openai.rs
use async_trait::async_trait;
use serde_json::json;
use crate::adapters::{BaseAdapter, ExecuteOptions, AdapterStream};
use crate::models::{Conversation, AdapterChatCompletion, Model, TokenUsage};
use crate::error::AdapterError;
use crate::http::ClientCache;

pub struct OpenAIAdapter {
    model: Model,
    api_key: String,
    base_url: String,
}

impl OpenAIAdapter {
    pub fn new(model: &Model, api_key: String) -> Self {
        Self {
            model: model.clone(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }
    
    fn transform_messages(&self, conversation: &Conversation) -> Result<Vec<serde_json::Value>, AdapterError> {
        // Apply model-specific message transformations based on capabilities
        let mut messages = Vec::new();
        
        for turn in &conversation.turns {
            // Transform turn to OpenAI message format
            let message = self.turn_to_message(turn)?;
            messages.push(message);
        }
        
        // Apply conversation constraints
        self.apply_constraints(&mut messages)?;
        
        Ok(messages)
    }
    
    fn apply_constraints(&self, messages: &mut Vec<serde_json::Value>) -> Result<(), AdapterError> {
        // Handle supports_repeating_roles
        if !self.model.capabilities.supports_repeating_roles {
            self.merge_repeating_roles(messages);
        }
        
        // Handle supports_system
        if !self.model.capabilities.supports_system {
            self.convert_system_to_user(messages);
        }
        
        // Handle supports_last_assistant
        if !self.model.capabilities.supports_last_assistant {
            if let Some(last) = messages.last() {
                if last["role"] == "assistant" {
                    messages.push(json!({"role": "user", "content": ""}));
                }
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl BaseAdapter for OpenAIAdapter {
    fn get_model(&self) -> &Model {
        &self.model
    }
    
    fn set_api_key(&mut self, api_key: String) -> Result<(), AdapterError> {
        self.api_key = api_key;
        Ok(())
    }
    
    async fn execute(
        &self,
        conversation: &Conversation,
        options: &ExecuteOptions,
    ) -> Result<AdapterChatCompletion, AdapterError> {
        let client = ClientCache::get_or_create(&self.base_url, &self.api_key);
        let messages = self.transform_messages(conversation)?;
        
        let mut body = json!({
            "model": self.model.name,
            "messages": messages,
        });
        
        // Add optional parameters
        if let Some(temp) = options.temperature {
            body["temperature"] = json!(temp);
        }
        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }
        if let Some(ref tools) = options.tools {
            body["tools"] = json!(tools);
        }
        
        let response = client
            .inner()
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        // Extract and calculate cost
        let usage: TokenUsage = serde_json::from_value(response["usage"].clone())?;
        let cost = self.model.cost.calculate(usage.prompt_tokens, usage.completion_tokens);
        
        let mut completion: AdapterChatCompletion = serde_json::from_value(response)?;
        completion.cost = cost;
        
        Ok(completion)
    }
    
    async fn execute_stream(
        &self,
        conversation: &Conversation,
        options: &ExecuteOptions,
    ) -> Result<AdapterStream, AdapterError> {
        // Streaming implementation using reqwest::Response::bytes_stream()
        // and futures::stream combinators
        todo!("Streaming implementation")
    }
}
```

### 5.3 Provider-Specific Transformations

Different providers require different message transformations:

| Provider | Transformation Required |
|----------|------------------------|
| **Anthropic** | - System message must be separate parameter, not in messages array<br>- Images must be base64 with media_type<br>- Tool format differs from OpenAI |
| **Cohere** | - No repeating roles allowed<br>- Chat history format with preamble<br>- Temperature range [0, 2]<br>- Different tool call format |
| **Gemini** | - Uses "parts" array instead of "content"<br>- System instruction separate parameter<br>- Different cost calculation (tiered by context length) |
| **Azure** | - Requires azure_endpoint and api_version<br>- tool_choice "required" → "auto" conversion |
| **Groq** | - OpenAI-compatible but no system messages for some models |

These will be implemented in provider-specific adapter modules under `src/adapters/providers/`.

## 6. Implementation Phases

### Phase 1: Core Data Models & models.dev Integration (2 weeks)
**Goal**: Establish data structures and models.dev API client

- Implement Serde structs for all data models (Conversation, Model, Cost, etc.)
- Implement models.dev API types and fetching
- Implement vendor extraction logic and config loading
- Write unit tests for serialization/deserialization
- Document field mappings

**Deliverables**:
- `src/models/` complete
- `src/config/` with provider defaults and vendor mappings
- Unit tests for model conversion
- Example: Fetch models.dev and print all OpenAI models

### Phase 2: HTTP Client Layer (1 week)
**Goal**: Robust HTTP client with caching and error handling

- Implement reqwest-based HttpClient
- Implement client caching with DashMap
- Add timeout and connection pool configuration
- Implement error handling for HTTP failures
- Add retry logic for transient failures

**Deliverables**:
- `src/http/` complete
- Integration test for client caching
- Environment variable configuration support

### Phase 3: Base Adapter Trait & OpenAI Adapter (2 weeks)
**Goal**: Working OpenAI adapter as reference implementation

- Define BaseAdapter trait with execute() and execute_stream()
- Implement OpenAI adapter with full capability support
- Implement message transformation pipeline
- Implement cost calculation
- Add comprehensive error handling

**Deliverables**:
- `src/adapters/base.rs` trait definition
- `src/adapters/providers/openai.rs` full implementation
- Integration tests against OpenAI API (with VCR/mocking)
- Example: Simple OpenAI chat completion

### Phase 4: AdapterFactory (1 week)
**Goal**: Dynamic adapter registry and model lookup

- Implement AdapterFactory with models.dev initialization
- Implement provider adapter registration
- Implement model filtering
- Add model path lookup
- Thread-safe singleton pattern with RwLock

**Deliverables**:
- `src/adapters/factory.rs` complete
- Tests for model lookup and filtering
- Example: List all vision-enabled models

### Phase 5: Additional Providers (3-4 weeks)
**Goal**: Support major providers with diverse APIs

Priority order:
1. **Anthropic**: Different message format, system message handling
2. **Gemini**: Different request/response schema, tiered pricing
3. **Cohere**: Chat history format, preamble
4. **Azure**: OpenAI-compatible with quirks
5. **Groq/Fireworks/Together**: OpenAI-compatible variations

**Deliverables**:
- Provider adapters in `src/adapters/providers/`
- Provider-specific config in `config/`
- Integration tests per provider
- Documentation for each provider's quirks

### Phase 6: Streaming Support (2 weeks)
**Goal**: Async streaming for all providers

- Implement streaming using futures::Stream
- Handle SSE (Server-Sent Events) parsing
- Implement stream state tracking for cost calculation
- Add error recovery and stream cleanup
- Test streaming with all providers

**Deliverables**:
- `execute_stream()` implementation for all adapters
- Stream utilities in `src/utils/`
- Streaming integration tests
- Example: Streaming chat with cancellation

### Phase 7: Advanced Features (2-3 weeks)
**Goal**: Vision, tools, and edge cases

- Implement vision support with image preprocessing
- Implement tool calling for all supporting providers
- Handle function calling (deprecated but still used)
- Add dynamic cost calculation for tiered models
- Implement response_format (JSON mode)
- Add n>1 support

**Deliverables**:
- Vision support in `src/utils/images.rs`
- Tool calling examples
- JSON mode tests
- Comprehensive capability validation

### Phase 8: Documentation & Publishing (1 week)
**Goal**: Production-ready crate

- Write comprehensive README.md
- Add rustdoc comments to all public APIs
- Create examples/ directory with common use cases
- Write migration guide from Python version
- Publish to crates.io

**Deliverables**:
- Complete documentation
- Published crate
- Examples for all major providers

## 7. Risks & Mitigation Strategies

### Risk 1: Incomplete models.dev Metadata
**Impact**: Missing capability flags lead to incorrect behavior  
**Mitigation**:
- Maintain provider-specific config files for capability defaults
- Add manual override mechanism for specific models
- Test against known model behaviors
- Fallback to conservative defaults (assume limitations)

### Risk 2: Provider API Changes
**Impact**: Adapters break when provider APIs change  
**Mitigation**:
- Version provider adapters separately
- Add provider API version detection
- Comprehensive integration test suite with real API calls
- Monitor models.dev API for changes

### Risk 3: Streaming Complexity in Rust
**Impact**: Difficult to implement cross-provider streaming  
**Mitigation**:
- Use proven crates (futures, tokio-stream)
- Start with simple OpenAI SSE streaming
- Reuse stream parsing logic across providers
- Add extensive streaming tests

### Risk 4: Type Safety vs Flexibility
**Impact**: Rust's type system makes dynamic provider handling harder  
**Mitigation**:
- Use trait objects (Box<dyn BaseAdapter>) for dynamic dispatch
- Use serde_json::Value for provider-specific parameters
- Provide strongly-typed builders for common use cases
- Document type conversion patterns

### Risk 5: Cost Calculation Accuracy
**Impact**: Incorrect cost tracking loses user trust  
**Mitigation**:
- Cross-reference with Python implementation
- Add cost validation tests against known conversations
- Support manual cost override
- Log warnings for models with unusual cost structures

### Risk 6: Performance of models.dev Fetching
**Impact**: Slow startup if fetching on every run  
**Mitigation**:
- Cache models.dev response with TTL
- Support offline mode with bundled snapshot
- Lazy initialization of adapters
- Async initialization to avoid blocking

### Risk 7: Maintenance Burden
**Impact**: Keeping up with new models and providers  
**Mitigation**:
- Automated CI to check for models.dev changes
- Clear contribution guide for adding providers
- Modular provider architecture for easy additions
- Community involvement

## 8. Open Questions & Follow-Up Tasks

### Open Questions

1. **Cache Strategy**: Should models.dev response be cached locally? What TTL?
   - **Recommendation**: Cache with 24-hour TTL, support manual refresh

2. **Vendor Extraction**: How to handle ambiguous model names?
   - **Recommendation**: Maintain curated mappings, allow manual override

3. **Backward Compatibility**: Should we maintain Python parity?
   - **Recommendation**: Match Python behavior but use Rust idioms

4. **Error Recovery**: How aggressive should retry logic be?
   - **Recommendation**: Configurable retries, default to 3 with exponential backoff

5. **Streaming Cancellation**: How to handle client-side stream cancellation?
   - **Recommendation**: Use Drop trait for cleanup, support tokio::select!

6. **Testing Strategy**: Use VCR-style mocking or real API calls?
   - **Recommendation**: Both - unit tests with mocks, integration tests with real API (opt-in)

7. **Async Runtime**: Require tokio or support multiple runtimes?
   - **Recommendation**: Tokio-only initially, abstract later if needed

8. **Configuration Format**: TOML vs JSON vs code?
   - **Recommendation**: TOML for static config, JSON for runtime, Rust builders for programmatic

### Follow-Up Tasks

- [ ] **Prototype models.dev parsing**: Validate field mapping with all providers
- [ ] **Performance benchmarks**: Compare Rust vs Python for throughput and latency
- [ ] **Security audit**: API key handling, sensitive data in logs
- [ ] **Rate limiting**: Add client-side rate limiting per provider
- [ ] **Observability**: Add tracing/logging with `tracing` crate
- [ ] **Cost tracking**: Persistent cost logging for accounting
- [ ] **Provider logos**: Fetch and cache SVG logos from models.dev
- [ ] **Model search**: Fuzzy search for models by name/capability
- [ ] **Prompt caching**: Support Anthropic/Claude prompt caching
- [ ] **Multi-turn conversations**: Optimize for conversation history management
- [ ] **Provider health checks**: Ping provider endpoints on startup
- [ ] **CLI tool**: Command-line interface for testing adapters

## 9. Assumptions

1. **models.dev Stability**: The models.dev API structure remains relatively stable
2. **Provider Parity**: New Rust port aims for feature parity with Python, not enhancement
3. **OpenAI Compatibility**: Most providers follow OpenAI's API conventions with variations
4. **Tokio Ecosystem**: Async Rust ecosystem (tokio, reqwest, futures) is mature enough
5. **Resource Constraints**: HTTP connection pooling is sufficient; no need for custom pooling
6. **API Keys**: Users provide API keys via environment variables or runtime configuration
7. **Internet Access**: Application has internet access to fetch models.dev API
8. **Model Turnover**: New models added frequently, old models deprecated slowly
9. **Cost Accuracy**: models.dev cost data is accurate and up-to-date
10. **Provider Coverage**: Supporting 5-10 major providers covers 95% of use cases

## 10. Success Criteria

The Rust port will be considered successful when:

1. ✅ **Feature Parity**: All features from Python version work in Rust
2. ✅ **Provider Coverage**: At least 5 major providers fully supported
3. ✅ **models.dev Integration**: Models automatically loaded from models.dev API
4. ✅ **Streaming Works**: Streaming works reliably for all supporting providers
5. ✅ **Performance**: Rust version matches or exceeds Python performance
6. ✅ **Documentation**: Comprehensive docs and examples available
7. ✅ **Testing**: >80% test coverage, all providers integration-tested
8. ✅ **Production Ready**: Used in at least one production system without issues
9. ✅ **Community**: Active contributors and responsive issue handling
10. ✅ **Maintenance**: Clear process for adding new models/providers

## Conclusion

This Rust port design leverages the models.dev API to create a maintainable, scalable adapter library for LLM chat completions. By separating model metadata (fetched dynamically) from provider-specific behaviors (implemented in adapters), we achieve a balance between flexibility and type safety.

The phased implementation approach allows for incremental development and validation, starting with core functionality (OpenAI) and expanding to diverse providers. The hybrid configuration strategy (models.dev + provider configs) ensures we can handle both common patterns and provider-specific quirks.

Key innovations over the Python version:
- **Dynamic model registry** from models.dev API
- **Strongly-typed Rust** for better compile-time guarantees
- **Async-first design** with tokio
- **Modular provider architecture** for easy extension

This design provides a clear roadmap for the Rust port while maintaining the flexibility to adapt to unexpected challenges during implementation.
