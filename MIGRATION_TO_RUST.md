# Migration to Pure Rust

This project has been migrated from a hybrid Python/Rust implementation to a pure Rust implementation.

## What Was Removed

### Python Adapters Package
- Removed entire `/adapters` directory containing Python implementation
- Removed Python test suite (`/tests/adapters`)
- Removed Python build files:
  - `pyproject.toml`
  - `poetry.lock`
  - `pytest.ini`
  - `.mypy.ini`
- Removed Python documentation generator (`docs/generate.py`)

### Updated Files
- Updated `README.md` to focus on Rust implementation
- Updated `.pre-commit-config.yaml` to use Rust tooling instead of Python linters

## What Was Added

### Models.dev API Configuration
Downloaded and converted the models.dev API data into multiple configuration formats:

1. **`config/models_api.json`** (379KB)
   - Full JSON response from https://models.dev/api.json
   - Contains all provider and model metadata

2. **`config/models_api.toml`** (383KB)
   - Full TOML conversion with all provider and model details
   - Includes all metadata fields

3. **`config/pure_api_config.toml`** (35KB)
   - Provider-level configuration only
   - Contains: id, name, env vars, npm package, API endpoint, documentation URL
   - Lists available models per provider

4. **`config/paras_config.toml`** (395KB)
   - Model parameters configuration
   - Contains detailed model capabilities, costs, limits, and metadata
   - Each model entry includes provider reference

## Configuration Structure

### Provider Configuration (`pure_api_config.toml`)
```toml
[provider_id]
id = "provider_id"
name = "Provider Name"
env = ["API_KEY_ENV_VAR"]
api = "https://api.provider.com/v1"
doc = "https://docs.provider.com"
models = ["model-1", "model-2", ...]
```

### Model Parameters (`paras_config.toml`)
```toml
[[provider_id.models]]
id = "model-id"
name = "Model Name"
attachment = false
reasoning = true
temperature = true
tool_call = true
modalities = { input = ["text"], output = ["text"] }
open_weights = false
cost = { input = 0.5, output = 1.0 }
limit = { context = 128000, output = 4096 }
provider = "provider_id"
```

## Rust Implementation

The Rust implementation continues to use:
- Dynamic model loading from models.dev API at runtime
- Provider capability defaults from `config/provider_defaults.toml`
- Vendor name extraction from `config/vendor_mappings.toml`
- New offline configuration files as fallback or reference

## Building and Testing

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run examples
cargo run --example list_models
cargo run --example filter_models

# Format and lint
cargo fmt
cargo clippy
```

## Migration Benefits

1. **Single Language**: Pure Rust implementation for better performance and type safety
2. **Offline Capability**: Local configuration files available as fallback
3. **Simplified Tooling**: No Python dependencies or toolchain needed
4. **Better Performance**: Rust's async/await and zero-cost abstractions
5. **Type Safety**: Compile-time guarantees for all model operations

## Providers Supported

The configuration includes 54 LLM providers from models.dev:
- OpenAI, Anthropic, Google Gemini, Cohere
- Amazon Bedrock, Azure, Groq, Fireworks
- Cerebras, Together, Perplexity, DeepSeek
- And 40+ more providers

## Next Steps

With the migration complete, development can focus on:
1. Implementing provider-specific adapters in Rust
2. Adding streaming support
3. Implementing vision and tool calling
4. Comprehensive testing suite
5. Performance optimization
