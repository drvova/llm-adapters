# Getting Started with Rust Port

## Quick Start

### Prerequisites

- Rust 1.90.0 or later
- Internet connection (for models.dev API)

### Installation

The Rust crate is located in the same repository as the Python package. All Rust code is in:
- `src/` - Rust source code
- `Cargo.toml` - Rust package manifest
- `config/` - Configuration files
- `examples/` - Example programs
- `tests/` - Test suite

### First Steps

1. **Build the project:**
   ```bash
   cargo build
   ```

2. **Run tests to verify everything works:**
   ```bash
   cargo test
   ```

3. **Try the examples:**
   ```bash
   # List all available models
   cargo run --example list_models
   
   # Filter models by capabilities
   cargo run --example filter_models
   
   # Build conversations
   cargo run --example conversation_builder
   ```

## Basic Usage

### 1. Initialize and List Models

```rust
use martian_adapters::{AdapterFactory, ModelFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize from models.dev
    AdapterFactory::init_from_modelsdev().await?;
    
    // Get all models
    let all_models = AdapterFactory::get_supported_models(None).await;
    println!("Total models: {}", all_models.len());
    
    // Get vision-enabled models
    let vision_models = AdapterFactory::get_supported_models(
        Some(ModelFilter::new().with_vision(true))
    ).await;
    println!("Vision models: {}", vision_models.len());
    
    Ok(())
}
```

### 2. Get Model Information

```rust
use martian_adapters::AdapterFactory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    AdapterFactory::init_from_modelsdev().await?;
    
    // Get a specific model
    let model = AdapterFactory::get_model("openai/openai/gpt-4o-mini").await?;
    
    println!("Model: {}", model.name);
    println!("Provider: {}", model.provider_name);
    println!("Context length: {}", model.context_length);
    println!("Supports vision: {}", model.capabilities.supports_vision);
    println!("Supports tools: {}", model.capabilities.supports_tools);
    println!("Cost per prompt token: ${:.6}", model.cost.prompt);
    
    Ok(())
}
```

### 3. Build Conversations

```rust
use martian_adapters::{
    Conversation, Turn, ConversationRole, TurnType,
};

fn main() {
    let mut conversation = Conversation::new();
    
    // Add a system message
    conversation.add_turn(TurnType::Basic(Turn {
        role: ConversationRole::System,
        content: "You are a helpful assistant.".to_string(),
    }));
    
    // Add a user message
    conversation.add_turn(TurnType::Basic(Turn {
        role: ConversationRole::User,
        content: "Hello!".to_string(),
    }));
    
    println!("Conversation has {} turns", conversation.len());
}
```

### 4. Filter Models

```rust
use martian_adapters::{AdapterFactory, ModelFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    AdapterFactory::init_from_modelsdev().await?;
    
    // Filter by multiple criteria
    let filtered = AdapterFactory::get_supported_models(
        Some(ModelFilter::new()
            .with_vision(true)
            .with_tools(true)
            .with_provider("openai".to_string()))
    ).await;
    
    for model in filtered {
        println!("- {}", model.name);
    }
    
    Ok(())
}
```

## Configuration

### Environment Variables

```bash
# HTTP Client Configuration
export ADAPTERS_MAX_CONNECTIONS_PER_PROCESS=1000
export ADAPTERS_MAX_KEEPALIVE_CONNECTIONS_PER_PROCESS=100
export ADAPTERS_HTTP_TIMEOUT=600
export ADAPTERS_HTTP_CONNECT_TIMEOUT=5

# API Keys (for future adapter implementations)
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
export COHERE_API_KEY=...

# Base URL Override (for testing)
export _ADAPTERS_OVERRIDE_ALL_BASE_URLS_="https://your-proxy.com/api"
```

### Provider Defaults

Provider-specific capability defaults are in `config/provider_defaults.toml`. You can modify this file to adjust default capabilities for each provider.

### Vendor Mappings

Model name to vendor mappings are configured in `config/vendor_mappings.toml` using regex patterns.

## Development

### Project Structure

```
martian-adapters-rust/
├── Cargo.toml                  # Dependencies and metadata
├── src/
│   ├── lib.rs                  # Main library entry point
│   ├── models/                 # Data structures
│   │   ├── conversation.rs     # Turn and Conversation types
│   │   ├── model.rs            # Model metadata
│   │   ├── cost.rs             # Cost tracking
│   │   ├── modelsdev.rs        # models.dev API types
│   │   └── response.rs         # Response types
│   ├── adapters/
│   │   ├── base.rs             # BaseAdapter trait
│   │   └── factory.rs          # Model registry and lookup
│   ├── http/
│   │   ├── client.rs           # HTTP client
│   │   └── cache.rs            # Client caching
│   ├── config/                 # Configuration
│   └── utils/                  # Utilities
├── config/
│   ├── provider_defaults.toml  # Provider capabilities
│   └── vendor_mappings.toml    # Vendor patterns
├── examples/                   # Example programs
└── tests/                      # Integration tests
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_conversation_creation

# Run with output
cargo test -- --nocapture

# Run in parallel (default)
cargo test
```

### Code Quality

```bash
# Check for compilation errors
cargo check

# Run linter
cargo clippy

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Adding Dependencies

Edit `Cargo.toml`:

```toml
[dependencies]
new-crate = "1.0"
```

Then run:
```bash
cargo build
```

## Common Patterns

### Error Handling

```rust
use martian_adapters::{AdapterError, Result};

async fn my_function() -> Result<()> {
    let model = AdapterFactory::get_model("invalid/path")
        .await
        .map_err(|e| {
            eprintln!("Error: {}", e);
            e
        })?;
    
    Ok(())
}
```

### Async/Await

All I/O operations are async and require Tokio:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your async code here
    Ok(())
}
```

Or in a library:

```rust
use tokio;

#[tokio::test]
async fn test_something() {
    // Test code
}
```

## Current Limitations

⚠️ **Phase 1 Complete** - The following features are not yet implemented:

- [ ] Provider adapters (OpenAI, Anthropic, etc.)
- [ ] Streaming responses
- [ ] Actual API calls to LLM providers
- [ ] Vision support (data structures ready, no API integration)
- [ ] Tool calling (data structures ready, no API integration)

These will be implemented in subsequent phases.

## Next Steps

1. **Explore Examples**: Run all examples to see what's possible
2. **Read Tests**: Check `tests/integration_test.rs` for usage patterns
3. **Check Design Doc**: Read `docs/rust_port_design.md` for architecture details
4. **Contribute**: Help implement Phase 2 (provider adapters)

## Getting Help

- Read the main Python documentation: `README.md`
- Check the Rust-specific README: `README_RUST.md`
- Review the design document: `docs/rust_port_design.md`
- Check implementation status: `RUST_PORT_STATUS.md`

## Performance Tips

1. **Initialize Once**: Call `init_from_modelsdev()` once at startup
2. **Reuse Clients**: The HTTP client cache is automatic
3. **Filter Early**: Use `ModelFilter` to reduce iteration over models
4. **Async Runtime**: Use Tokio for best performance

## Troubleshooting

### Compilation Errors

```bash
# Clean and rebuild
cargo clean
cargo build
```

### Network Issues

The `init_from_modelsdev()` call requires internet access to fetch model data from https://models.dev/api.json. Ensure:
- Internet connection is available
- No firewall blocking HTTPS requests
- models.dev is accessible

### Missing Models

If a model path doesn't work:
1. List all models: `cargo run --example list_models`
2. Check the correct path format: `provider/vendor/model-name`
3. Verify the model exists in models.dev

## Contributing

To contribute to the Rust port:

1. Read the design document
2. Pick a task from the roadmap in `RUST_PORT_STATUS.md`
3. Implement with tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## License

Apache-2.0 (same as Python package)
