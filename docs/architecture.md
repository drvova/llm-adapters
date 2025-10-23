# Python Adapter Architecture

## Overview

The `martian-adapters` package (published from the `adapters` module) provides a unified chat-completions interface across many LLM providers. It enables applications to interact with different LLM APIs through a consistent OpenAI-style schema, abstracting away provider-specific differences in request/response formats, capabilities, and SDK interfaces.

**Key Principle**: The package talks directly to provider SDKs and has **no models.dev integration** today. Each provider adapter wraps the provider's official Python SDK (OpenAI, Anthropic, Cohere, Google Generative AI, etc.) and normalizes conversations, responses, and capabilities to a common interface.

## Architecture Components

### 1. Type System (`types.py`)

The foundation of the adapter system is a rich Pydantic-based type system that models conversations, models, costs, and responses.

#### Core Types

**Enums:**
- `Provider`: Enumeration of supported providers (openai, anthropic, azure, cohere, gemini, groq, fireworks, together, etc.)
- `Vendor`: Model vendors (meta-llama, openai, anthropic, cohere, google, mistralai, etc.)
- `ConversationRole`: Message roles (user, assistant, system, function, tool)
- `AdapterFinishReason`: Completion reasons (stop, length, tool_calls, content_filter, function_call)

**Turn Types:**
- `Turn`: Basic message with role and content
- `ContentTurn`: Multi-modal message supporting text and images
- `FunctionOutputTurn` / `ToolOutputTurn`: Function/tool response messages
- `FunctionCallTurn` / `ToolsCallTurn`: Function/tool invocation messages
- `TurnType`: Union of all turn types

**Conversation:**
- `Conversation`: Ordered list of turns representing a chat history
- Provides list-like interface (`__getitem__`, `__iter__`, `__len__`)
- Helper method `is_last_turn_vision_query()` for vision detection

**Model Metadata:**
```python
class Model(BaseModel):
    name: str
    vendor_name: str
    provider_name: str
    cost: Cost  # prompt, completion, request costs per token
    context_length: int
    completion_length: Optional[int]
    
    # Capability flags (all default to True unless overridden)
    supports_user: bool = True
    supports_repeating_roles: bool = True
    supports_streaming: bool = True
    supports_vision: bool = True
    supports_tools: bool = True
    supports_n: bool = True
    supports_system: bool = True
    supports_multiple_system: bool = True
    supports_empty_content: bool = True
    supports_tool_choice: bool = True
    supports_tool_choice_required: bool = True
    supports_json_output: bool = True
    supports_json_content: bool = True
    supports_last_assistant: bool = True
    supports_first_assistant: bool = True
    supports_temperature: bool = True
    supports_only_system: bool = True
    supports_only_assistant: bool = True
    
    properties: ModelProperties  # open_source, chinese, gdpr_compliant, is_nsfw
```

**Response Types:**
- `AdapterChatCompletion`: Extends OpenAI's `ChatCompletion` with cost tracking
- `AdapterChatCompletionChunk`: Streaming chunk wrapper
- `AdapterStreamSyncChatCompletion` / `AdapterStreamAsyncChatCompletion`: Stream wrappers

### 2. Abstract Adapters (`abstract_adapters/`)

The adapter hierarchy uses composition and mixins to provide reusable functionality.

#### `BaseAdapter` (base_adapter.py)

The root abstract class defining the adapter interface:

```python
class BaseAdapter(ABC):
    @abstractmethod
    def get_model(self) -> Model:
        pass
    
    @abstractmethod
    def execute_sync(self, llm_input: Conversation, stream: bool = False, **kwargs) -> AdapterChatCompletion | AdapterStreamSyncChatCompletion:
        pass
    
    @abstractmethod
    async def execute_async(self, llm_input: Conversation, stream: bool = False, **kwargs) -> AdapterChatCompletion | AdapterStreamAsyncChatCompletion:
        pass
    
    def set_api_key(self, api_key: str) -> None:
        pass
```

#### `ApiKeyAdapterMixin` (api_key_adapter_mixin.py)

Handles API key management with rotation support:

- Reads API keys from environment variables (e.g., `OPENAI_API_KEY`)
- Supports key lists via `{PROVIDER}_API_KEY_LIST` env vars (comma-separated)
- Implements round-robin key rotation for rate limit distribution
- Allows programmatic key setting via `set_api_key()`

```python
class ApiKeyAdapterMixin:
    _api_key: str
    _api_keys: List[str]
    _next_api_key: int
    
    def get_api_key_name() -> str:  # abstract, returns env var name
    def get_api_key() -> str:  # returns next key in rotation
    def set_api_key(self, api_key: str) -> None:
```

#### `ProviderAdapterMixin` (provider_adapter_mixin.py)

Enables adapters to support multiple models from a single provider:

```python
class ProviderAdapterMixin:
    _current_model: Optional[Model] = None
    
    def _set_current_model(self, model: Model) -> None:
        self._current_model = model
    
    @staticmethod
    @abstractmethod
    def get_supported_models() -> List[Model]:
        pass
```

#### `SDKChatAdapter` (sdk_chat_adapter.py)

The primary implementation class that most provider adapters inherit from. It's a generic class parameterized by sync and async client types:

```python
class SDKChatAdapter(
    BaseAdapter,
    ApiKeyAdapterMixin,
    ProviderAdapterMixin,
    Generic[CLIENT_SYNC, CLIENT_ASYNC]
):
```

**Responsibilities:**

1. **Client Management:**
   - Creates sync and async SDK clients
   - Uses `ClientCache` to reuse clients by (base_url, api_key, mode)
   - Configures HTTP settings (timeouts, connection pools)
   - Supports base URL overrides for testing

2. **Request Normalization (`_get_params`):**
   The most complex method, transforming OpenAI-style requests to provider requirements:
   
   - **Capability Validation**: Checks model flags and raises `AdapterException` for unsupported features
   - **Content Transformations**:
     - Converts JSON content arrays to strings if not supported
     - Replaces empty strings with `EMPTY_CONTENT = '""'` placeholder
     - Processes vision image URLs
   - **Role Transformations**:
     - Converts system roles to user/assistant if not supported
     - Limits multiple system messages to first only
     - Joins consecutive same-role messages if repeating roles not supported
     - Adds empty user messages before/after assistant messages as needed
   - **Parameter Filtering**:
     - Removes `user` field if not supported
     - Removes or validates `n`, `tools`, `functions`, `temperature`, `response_format`
     - Adjusts `max_tokens` against completion length limits
   
3. **Response Extraction:**
   - Abstract methods `_extract_response()` and `_extract_stream_response()`
   - Subclasses implement provider-specific response parsing
   - Calculates costs based on token usage and model pricing

4. **Streaming Support:**
   - Wraps sync/async generators
   - Handles chunk parsing via `_extract_stream_response()`
   - Auto-closes streams on completion or error
   - Maintains state across chunks (e.g., for tool call accumulation)

5. **Execution:**
   - `execute_sync()` / `execute_async()` orchestrate the flow:
     1. Normalize parameters via `_get_params()`
     2. Call SDK via `_call_sync()` / `_call_async()`
     3. Extract response or return stream generator
     4. Calculate costs and wrap in adapter types

**Abstract Methods to Implement:**
```python
def _create_client_sync(self, base_url: str, api_key: str) -> CLIENT_SYNC
def _create_client_async(self, base_url: str, api_key: str) -> CLIENT_ASYNC
def _call_sync(self) -> Callable  # returns SDK method reference
def _call_async(self) -> Callable
def get_base_sdk_url(self) -> str
def _extract_response(self, request: Any, response: Any) -> AdapterChatCompletion
def _extract_stream_response(self, request: Any, response: Any, state: dict) -> AdapterChatCompletionChunk
```

#### `OpenAISDKChatAdapter` (openai_sdk_chat_adapter.py)

A concrete implementation of `SDKChatAdapter` for OpenAI-compatible APIs:

- Uses `openai.OpenAI` / `openai.AsyncOpenAI` clients
- Configures `httpx` clients with connection limits and timeouts
- Extracts responses directly from OpenAI's response objects
- Handles reasoning tokens for o1 models
- Most provider adapters inherit from this (Groq, Fireworks, Together, Azure, etc.)

### 3. Provider Adapters (`provider_adapters/`)

Each provider module defines:

1. **Model List**: A `MODELS` list with cost, capability, and context length metadata
2. **Model Class**: Often a subclass of `Model` with provider defaults
3. **Adapter Class**: Subclass of `SDKChatAdapter` or `OpenAISDKChatAdapter`

**Patterns:**

**A. OpenAI-Compatible Providers** (inherit `OpenAISDKChatAdapter`):
- Fireworks, Groq, Cerebras, DeepInfra, Databricks, Lepton, Moonshot, OctoAI, Together, Azure, OpenRouter, Perplexity
- Only need to override:
  - `get_supported_models()` → return MODELS
  - `get_api_key_name()` → return env var name
  - `get_base_sdk_url()` → return API base URL
- Example (Groq):
  ```python
  class GroqSDKChatProviderAdapter(OpenAISDKChatAdapter):
      @staticmethod
      def get_supported_models() -> list[Model]:
          return MODELS
      
      @staticmethod
      def get_api_key_name() -> str:
          return "GROQ_API_KEY"
      
      def get_base_sdk_url(self) -> str:
          return "https://api.groq.com/openai/v1"
  ```

**B. Custom SDK Providers** (implement full `SDKChatAdapter`):
- **Anthropic**: Uses Anthropic SDK, converts messages to their format (system prompt separate, content blocks), handles streaming events
- **Cohere**: Uses Cohere SDK v2, converts to Cohere's message format, maps tool calls, handles preamble for system messages
- **Gemini**: Uses Google Generative AI SDK, converts to Gemini's content format, handles dynamic pricing based on context length
- **Vertex**: Similar to Gemini but for GCP Vertex AI

These adapters implement custom:
- Message format conversion (in `_get_params`)
- Response extraction (parsing provider-specific response objects)
- Stream chunk handling (provider-specific stream event types)
- Cost calculation (sometimes dynamic, like Gemini's tiered pricing)

**C. Model Metadata Definition**:

Each provider defines models with specific capability overrides:

```python
class AnthropicModel(Model):
    vendor_name: str = Vendor.anthropic.value
    provider_name: str = Provider.anthropic.value
    supports_vision: bool = False  # override default
    supports_empty_content: bool = False
    supports_n: bool = False
    supports_only_system: bool = False

MODELS: list[Model] = [
    AnthropicModel(
        name="claude-3-5-sonnet-20241022",
        cost=Cost(prompt=3.0e-6, completion=15.0e-6),
        context_length=200000,
        completion_length=4096,
    ),
    # ... more models
]
```

### 4. Adapter Factory (`adapter_factory.py`)

The `AdapterFactory` uses reflection to build registries of adapters and models at module load time.

**Registries:**

1. **Adapter Registry** (`_adapter_registry: dict[str, type[BaseAdapter]]`):
   - Maps model paths (e.g., `"fireworks/meta-llama/llama-v3p1-405b-instruct"`) to adapter classes
   - Populated by introspecting `sys.modules["adapters.provider_adapters"]`
   - Finds all classes that are subclasses of both `ProviderAdapterMixin` and `BaseAdapter`
   - For each model in `get_supported_models()`, registers `model.get_path() → adapter_class`
   - Special handling for OpenAI, Anthropic, Gemini (registered by model name, not path)

2. **Model Registry** (`_model_registry: dict[str, Model]`):
   - Maps model paths to `Model` instances
   - Same introspection logic as adapter registry

**Factory Methods:**

```python
AdapterFactory.get_adapter_by_path(model_path: str) -> BaseAdapter | None
    # Looks up adapter class, instantiates, sets current model
    
AdapterFactory.get_adapter(model: Model) -> BaseAdapter | None
    # Same but takes Model instance
    
AdapterFactory.get_model_by_path(model_path: str) -> Model | None
    # Returns Model metadata
    
AdapterFactory.get_supported_models(**filters) -> list[Model]
    # Returns all models, optionally filtered by capability flags
    # Respects ADAPTER_DISABLED_MODELS env var
```

**Model Path Format:**
- Most providers: `provider/vendor/model_name` (e.g., `groq/meta-llama/llama-3.1-70b-versatile`)
- OpenAI/Anthropic/Gemini: Just `model_name` (e.g., `gpt-4o`, `claude-3-5-sonnet-20241022`)

### 5. Supporting Infrastructure

#### `ClientCache` (client_cache.py)

A simple in-memory cache to reuse HTTP clients:

```python
class ClientCache:
    _client_cache: Dict[str, Any]
    
    def get_client(self, base_url: str, api_key: str, mode: Literal["sync", "async"]) -> Any
    def set_client(self, base_url: str, api_key: str, mode: Literal["sync", "async"], client: Any) -> None
```

- Key format: `f"{base_url}-{api_key}-{mode}"`
- Prevents creating redundant HTTP connections
- Note: No TTL/expiration currently (potential memory leak if many keys used)

#### `constants.py`

Environment-driven HTTP configuration:

```python
OVERRIDE_ALL_BASE_URLS = os.getenv("_ADAPTERS_OVERRIDE_ALL_BASE_URLS_")
MAX_CONNECTIONS_PER_PROCESS = int(os.getenv("ADAPTERS_MAX_CONNECTIONS_PER_PROCESS", "1000"))
MAX_KEEPALIVE_CONNECTIONS_PER_PROCESS = int(os.getenv("ADAPTERS_MAX_KEEPALIVE_CONNECTIONS_PER_PROCESS", "100"))
HTTP_TIMEOUT = float(os.getenv("ADAPTERS_HTTP_TIMEOUT", "600.0"))
HTTP_CONNECT_TIMEOUT = float(os.getenv("ADAPTERS_HTTP_CONNECT_TIMEOUT", "5.0"))
```

- Used when creating `httpx` clients for OpenAI-compatible adapters
- `OVERRIDE_ALL_BASE_URLS`: For testing/proxying, redirects all requests to a single base URL

#### `general_utils.py`

Utility functions:

- `delete_none_values(dict)`: Recursively removes None values from request dicts
- `process_image_url_anthropic(url)`: Converts image URLs (including data URIs) to Anthropic's base64 format
- `get_dynamic_cost(model_name, token_count)`: Calculates tiered pricing for Gemini models
- `stream_generator_auto_close`: Async context manager for stream cleanup
- `EMPTY_CONTENT = '""'`: Placeholder for empty content when provider doesn't support it

#### `custom_sdk_chat_provider_adapter.py`

Provides `CustomOpenAISDKChatProviderAdapter` for runtime base URL injection:

```python
class CustomOpenAISDKChatProviderAdapter(OpenAISDKChatAdapter):
    def __init__(self, base_url: str):
        self.base_url = base_url
        super().__init__()
    
    def get_base_sdk_url(self) -> str:
        return self.base_url
```

- Useful for connecting to custom OpenAI-compatible endpoints
- Not registered in AdapterFactory (must be instantiated directly)

### 6. Request/Response Normalization

The architecture centers on normalizing all provider interactions to the OpenAI schema.

**Request Flow:**

1. **Input**: `Conversation` with OpenAI-style turns
2. **Normalization** (`SDKChatAdapter._get_params`):
   - Serialize turns to dicts: `[turn.model_dump() for turn in llm_input.turns]`
   - Apply transformations based on model capabilities:
     - Role conversions (system → user/assistant)
     - Content format conversions (list → string)
     - Empty content handling
     - Repeating role consolidation
     - First/last message adjustments
   - Validate features (tools, vision, streaming, etc.)
   - Build final SDK-compatible params dict
3. **Provider Adapter** (if custom):
   - Further transform messages to provider format (e.g., Anthropic's content blocks)
4. **SDK Call**: Invoke provider SDK with normalized params
5. **Response Extraction**:
   - Parse provider response
   - Calculate token usage and cost
   - Map finish reasons to OpenAI equivalents
   - Wrap in `AdapterChatCompletion`
6. **Output**: OpenAI-style response with cost metadata

**Streaming Flow:**

Similar, but:
- Returns generator/async generator of `AdapterChatCompletionChunk`
- Each chunk parsed via `_extract_stream_response()`
- State dict passed through chunks for accumulation (e.g., tool calls)
- Auto-close on generator exit

**Cost Calculation:**

```python
cost = (
    model.cost.prompt * prompt_tokens +
    model.cost.completion * completion_tokens +
    model.cost.request  # fixed per-request cost
)
```

For Gemini, uses dynamic pricing based on context length tiers.

### 7. Test Infrastructure

The package uses pytest with VCR (pytest-recording) for deterministic testing.

**Test Strategy:**

1. **VCR Cassettes**:
   - Tests decorated with `@pytest.mark.vcr`
   - HTTP interactions recorded to YAML cassettes in `tests/adapters/cassettes/`
   - Headers filtered (authorization, api keys) via `vcr_config` fixture
   - Enables testing without live API keys
   - Cassettes checked into git for CI/CD

2. **Parametrized Tests**:
   - `ADAPTER_TEST_FACTORIES` list in `tests/utils.py`
   - Tests run against all supported model/adapter combinations
   - Example: `@pytest.mark.parametrize("factory", ADAPTER_TEST_FACTORIES)`

3. **Test Categories**:
   - **Conversation Tests** (`tests/adapters/conversation/`):
     - `first/`: First message role handling
     - `last/`: Last message role handling
     - `only/`: Single-message conversations (only_user, only_system, only_assistant)
     - `repeating/`: Consecutive same-role messages
     - `json/`: JSON content and output formats
   - **Feature Tests** (`tests/adapters/`):
     - `test_execute_stream.py`: Streaming responses
     - `test_execute_vision.py`: Vision/image inputs
     - `test_execute_n.py`: Multiple completions
     - `test_execute_temperature.py`: Temperature parameter
     - `test_execute_empty_content.py`: Empty message handling
   - **Tools Tests** (`tests/adapters/tools/`):
     - Function calling and tool usage

4. **Test Fixtures**:
   - `adapters_patch` fixture: Disables client caching during tests
   - `vcr_config` fixture: Configures header filtering
   - `SIMPLE_CONVERSATION_*` constants: Reusable test conversations

5. **Client Cache Patching**:
   ```python
   @pytest.fixture(name="adapters_patch", autouse=True)
   def fixture_adapters_patch(monkeypatch):
       monkeypatch.setattr(
           "adapters.client_cache.client_cache.get_client",
           lambda base_url, api_key, mode: None,
       )
   ```
   Forces fresh client creation in each test to avoid cross-test pollution.

**Running Tests:**

```bash
# Run all tests with cassettes
poetry run pytest

# Re-record cassettes (requires API keys)
poetry run pytest --record-mode=rewrite

# Run specific test category
poetry run pytest tests/adapters/conversation/only/

# Run in parallel
poetry run pytest -n auto
```

## Data Flow Diagram

```
User Code
  ↓
AdapterFactory.get_adapter_by_path(model_path)
  ↓
[Registry Lookup] → Adapter Instance + Model Metadata
  ↓
adapter.execute_sync(conversation, **kwargs)
  ↓
SDKChatAdapter._get_params(conversation, **kwargs)
  ↓
[Message Normalization + Validation]
  ↓
Provider SDK Client (cached or created)
  ↓
[HTTP Request] → Provider API
  ↓
Provider SDK Response
  ↓
_extract_response() or _extract_stream_response()
  ↓
[Cost Calculation + Response Wrapping]
  ↓
AdapterChatCompletion (with cost, token counts, OpenAI schema)
  ↓
User Code
```

## Configuration

**Environment Variables:**

**API Keys:**
- `{PROVIDER}_API_KEY`: Single API key (e.g., `OPENAI_API_KEY`)
- `{PROVIDER}_API_KEY_LIST`: Comma-separated keys for rotation

**HTTP Settings:**
- `ADAPTERS_MAX_CONNECTIONS_PER_PROCESS`: Default 1000
- `ADAPTERS_MAX_KEEPALIVE_CONNECTIONS_PER_PROCESS`: Default 100
- `ADAPTERS_HTTP_TIMEOUT`: Request timeout in seconds, default 600
- `ADAPTERS_HTTP_CONNECT_TIMEOUT`: Connection timeout in seconds, default 5

**Testing/Debugging:**
- `_ADAPTERS_OVERRIDE_ALL_BASE_URLS_`: Redirect all requests to a single URL
- `ADAPTER_DISABLED_MODELS`: Comma-separated model names to exclude

**Backward Compatibility:**
- Also checks `MAX_CONNECTIONS_PER_PROCESS`, `HTTP_TIMEOUT`, `HTTP_CONNECT_TIMEOUT` (without `ADAPTERS_` prefix)

## Key Design Patterns

1. **Composition over Inheritance**: Mixins (`ApiKeyAdapterMixin`, `ProviderAdapterMixin`) provide reusable behaviors

2. **Adapter Pattern**: Each provider adapter translates between OpenAI schema and provider SDK

3. **Registry Pattern**: `AdapterFactory` maintains mappings for dynamic lookup

4. **Template Method**: `SDKChatAdapter` defines execution flow, subclasses implement SDK-specific details

5. **Strategy Pattern**: Different providers = different strategies for API interaction

6. **Generics**: `SDKChatAdapter[CLIENT_SYNC, CLIENT_ASYNC]` parameterized by client types

7. **Capability Flags**: Model metadata drives runtime behavior (avoid errors by validating upfront)

8. **Client Caching**: Singleton-like cache for HTTP clients to reduce overhead

9. **Normalize-Execute-Denormalize**: Transform input → call provider → transform output

## Provider Integration Summary

**Total Providers**: 17+

**OpenAI-Compatible** (11):
- Azure, Cerebras, Databricks, DeepInfra, Fireworks, Groq, Lepton, Moonshot, OctoAI, OpenRouter, Perplexity, Together

**Custom SDK** (6):
- OpenAI (native), Anthropic, Cohere, Gemini, Vertex, AI21

**Model Count**: 100+ models across all providers

**Common Capabilities**:
- Streaming: Most models
- Tools: Many models (GPT-4, Claude 3+, Gemini, etc.)
- Vision: Select models (GPT-4o, Claude 3, Gemini, Llama 3.2 vision variants)
- JSON Output: Most models
- Temperature Control: Most models (except o1 series)

## Notable Implementation Details

1. **Message Role Handling**: Complex logic in `_get_params` to handle 15+ capability combinations (supports_system, supports_repeating_roles, supports_first_assistant, etc.)

2. **Cost Tracking**: Calculated at response time using token counts and model metadata; stored in `AdapterChatCompletion.cost`

3. **Streaming State**: `_extract_stream_response` receives mutable `state` dict to accumulate partial data (e.g., tool call arguments across chunks)

4. **Image Processing**: For Anthropic, images fetched via httpx and base64-encoded; other providers pass URLs directly

5. **Empty Content**: Uses `'""'` string literal as placeholder for providers that reject empty strings

6. **API Key Rotation**: Round-robin across key list, useful for rate limit distribution

7. **Error Handling**: Raises `AdapterException` for unsupported features; providers may raise `AdapterRateLimitException`

8. **No Retry Logic**: `max_retries=0` in OpenAI SDK clients; caller responsible for retries

9. **Sync/Async Parity**: All adapters must implement both `execute_sync` and `execute_async`

10. **Model Path Inconsistency**: OpenAI/Anthropic/Gemini use model name only; others use provider/vendor/model format (historical quirk)

## Current Limitations & Notes

1. **No models.dev Integration**: The package does **not** integrate with any models.dev service or API. All communication is direct to provider SDKs.

2. **Client Cache Memory**: No TTL on cached clients; potential memory leak with many API keys

3. **Single Model per Request**: Cannot mix models in a single conversation

4. **No Automatic Retry**: Caller must handle rate limits and transient errors

5. **Prompt Caching**: Not exposed (some providers like Anthropic support it but adapter doesn't surface it)

6. **Batch Requests**: Not supported (each `execute_*` is a single request)

7. **Fine-tuned Models**: Supported if provider uses same format (just pass model name), but no special handling

8. **Cost Precision**: Uses provider-published costs; actual billing may differ

9. **Dynamic Model Discovery**: Registry populated at import time; new models require code update

10. **Type Checking**: Uses Pydantic for runtime validation; MyPy support via `.mypy.ini`

## Package Dependencies

**Core:**
- `pydantic ^2.9.2`: Type system and validation
- `openai ^1.54.1`: OpenAI SDK (used by 11+ providers)
- `anthropic ^0.39.0`: Anthropic SDK
- `cohere ^5.11.3`: Cohere SDK
- `google-generativeai ^0.8.3`: Gemini SDK
- `brotli ^1.1.0`: HTTP compression

**Dev/Test:**
- `pytest`, `pytest-asyncio`, `pytest-recording`: Testing framework
- `vcrpy ^6.0.2`: HTTP interaction recording
- `mypy ^1.13.0`: Type checking
- `ruff ^0.7.2`: Linting/formatting
- `pre-commit ^4.0.1`: Git hooks

## Rust Port Considerations

When porting this architecture to Rust, consider:

1. **SDK Availability**: Not all providers have Rust SDKs; may need to implement HTTP clients directly

2. **Async Runtime**: Choose tokio or async-std; ensure compatibility with all provider SDKs

3. **Type System**: Rust's type system is stricter; use enums for turn types, trait objects for adapters

4. **Error Handling**: Replace `AdapterException` with Result types; consider custom error enums

5. **Client Caching**: Use `Arc<Mutex<HashMap>>` or `DashMap` for thread-safe caching

6. **Streaming**: Use async streams (`futures::Stream`) instead of generators

7. **Message Normalization**: Port ~300 lines of Python logic in `_get_params`; critical for correctness

8. **Reflection**: Rust doesn't have runtime reflection; consider explicit registration or macros

9. **Testing**: Use `vcr-cassette` crate or similar; may need to adapt cassette format

10. **Performance**: Rust will be faster; opportunity to optimize hot paths (message normalization, JSON parsing)

11. **Backward Compatibility**: Maintain OpenAI schema and model path conventions for drop-in replacement

12. **Pydantic Equivalent**: Consider `serde` for serialization; may need custom validation

13. **HTTP Clients**: Use `reqwest` or `hyper`; configure connection pooling, timeouts

14. **Environment Config**: Use `dotenv` or similar; maintain same env var names

15. **Cost Calculation**: Port dynamic pricing logic (Gemini); ensure floating-point precision

## Further Reading

- **README.md**: Package installation, quickstart, contributing guidelines
- **docs/index.md**: Complete model capability matrix (100+ models)
- **docs/generate.py**: Script to regenerate capability matrix from model metadata
- **.env.example**: All supported environment variables
- **tests/utils.py**: Test utilities, example conversations
- **pyproject.toml**: Package metadata, version (7.0.0a2), dependencies

## Summary

The Python adapter architecture is a layered, extensible system for unifying LLM provider interactions:

- **Core abstraction**: OpenAI-style `Conversation` → provider SDK → OpenAI-style response
- **Key components**: Type system, abstract adapters, provider adapters, factory, support infrastructure
- **Primary complexity**: Message normalization logic to handle diverse provider requirements
- **Testing approach**: VCR cassettes for deterministic, offline testing
- **Design philosophy**: Direct SDK usage, composition via mixins, capability-driven behavior
- **Current scope**: Python package only; no models.dev integration; ~100 models across 17+ providers

This document serves as the reference for understanding the existing Python implementation and planning the Rust port effort.
