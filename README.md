# Martian Adapters (Rust)

Unified chat-completions interface across many LLM providers, written in Rust.

## Overview

The Martian Adapters package facilitates communication between different language model APIs by providing a unified interface for interaction. This ensures ease of use and flexibility in integrating multiple models from various providers.

## Features

- 🌐 **Multi-Provider Support**: Unified interface for OpenAI, Anthropic, Google Gemini, Cohere, and 50+ other providers
- 📊 **Dynamic Model Discovery**: Automatically fetches model metadata from [models.dev](https://models.dev) API
- 💰 **Cost Tracking**: Accurate token usage and cost calculation per request
- ⚡ **Async/Await**: Built on Tokio for high-performance async operations
- 🔧 **Capability Detection**: Automatic detection of model capabilities (vision, tools, streaming, etc.)
- 🛡️ **Type Safety**: Strongly-typed Rust API with comprehensive error handling

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
martian-adapters = "0.1.0"
```

## Quick Start

```rust
use martian_adapters::{
    AdapterFactory, Conversation, Turn, ConversationRole,
    TurnType, ExecuteOptions, ModelFilter
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the factory with models.dev data
    AdapterFactory::init_from_modelsdev().await?;

    // Get a model
    let model = AdapterFactory::get_model("openai/openai/gpt-4o-mini").await?;
    println!("Model: {}", model.name);
    println!("Context length: {}", model.context_length);
    println!("Supports vision: {}", model.capabilities.supports_vision);

    // List all vision-enabled models
    let vision_models = AdapterFactory::get_supported_models(
        Some(ModelFilter::new().with_vision(true))
    ).await;
    println!("Vision-enabled models: {}", vision_models.len());

    Ok(())
}
```

## Configuration

Environment variables (optional):

```bash
# HTTP Client Configuration
ADAPTERS_MAX_CONNECTIONS_PER_PROCESS=1000
ADAPTERS_MAX_KEEPALIVE_CONNECTIONS_PER_PROCESS=100
ADAPTERS_HTTP_TIMEOUT=600
ADAPTERS_HTTP_CONNECT_TIMEOUT=5

# Base URL Override (for testing)
_ADAPTERS_OVERRIDE_ALL_BASE_URLS_="https://your-proxy.com/api"

# Provider API Keys
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
COHERE_API_KEY=...
```

Copy `.env.example` to `.env` and populate it with appropriate values.

## Development

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run with example
cargo run --example list_models

# Check for issues
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

### Setting Up Pre-commit

```bash
pre-commit install
```

To run pre-commit manually:

```bash
pre-commit run --all-files
```

## Examples

See the `examples/` directory for more examples:

```bash
# List all available models
cargo run --example list_models

# Filter models by capability
cargo run --example filter_models
```

## Architecture

The Rust implementation uses a hybrid approach:

1. **models.dev API**: Primary source for model metadata (costs, context lengths, capabilities)
2. **Provider Configuration Files**: TOML files defining provider-specific capability defaults
3. **Vendor Mappings**: Regex patterns to extract vendor names from model IDs
4. **Adapter Trait**: Common interface for all providers

## Current Status

### ✅ Completed (Phase 1)

- Core data models (Conversation, Turn, Model, Cost, etc.)
- models.dev API integration and parsing
- Provider capability defaults configuration
- Vendor name extraction and mapping
- HTTP client layer with caching
- Error handling infrastructure
- Configuration management (environment variables)
- Basic factory pattern for model lookup and filtering

### 🚧 In Progress

- Provider adapters implementation (OpenAI, Anthropic, etc.)
- Streaming support
- Vision and tool calling support

### 📋 Planned

- Full adapter implementations for all major providers
- Streaming responses
- Tool/function calling
- Vision support with image preprocessing
- Comprehensive testing suite

## Related Projects

- [models.dev](https://models.dev) - LLM model metadata aggregator

## License

Apache-2.0

## Contributing

This is an active project. Contributions welcome!

1. Check the [design document](docs/rust_port_design.md) for architecture details
2. Pick a task from the implementation phases
3. Submit a PR with tests

## Support

For issues and questions:
- Open an issue on GitHub
- See the [Getting Started Guide](GETTING_STARTED_RUST.md)
