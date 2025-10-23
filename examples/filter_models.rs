use martian_adapters::{AdapterFactory, ModelFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing from models.dev...\n");
    AdapterFactory::init_from_modelsdev().await?;

    println!("🔍 Filtering Examples:\n");

    let openai_models = AdapterFactory::get_supported_models(Some(
        ModelFilter::new().with_provider("openai".to_string()),
    ))
    .await;
    println!("OpenAI models: {}", openai_models.len());
    for model in openai_models.iter().take(5) {
        println!("  - {} (context: {})", model.name, model.context_length);
    }

    println!();

    let vision_tool_models = AdapterFactory::get_supported_models(Some(
        ModelFilter::new().with_vision(true).with_tools(true),
    ))
    .await;
    println!("Models with vision AND tools: {}", vision_tool_models.len());
    for model in vision_tool_models.iter().take(5) {
        println!(
            "  - {}/{}/{}",
            model.provider_name, model.vendor_name, model.name
        );
    }

    println!();

    let anthropic_models = AdapterFactory::get_supported_models(Some(
        ModelFilter::new().with_provider("anthropic".to_string()),
    ))
    .await;
    println!("Anthropic models: {}", anthropic_models.len());
    for model in anthropic_models.iter() {
        let cost_per_1m_prompt = model.cost.prompt * 1_000_000.0;
        let cost_per_1m_completion = model.cost.completion * 1_000_000.0;
        println!(
            "  - {} (${:.2}/${:.2} per 1M tokens)",
            model.name, cost_per_1m_prompt, cost_per_1m_completion
        );
    }

    println!();

    if let Ok(model) =
        AdapterFactory::get_model("anthropic/anthropic/claude-3-5-sonnet-20241022").await
    {
        println!("📊 Detailed Model Info: {}", model.name);
        println!("  Provider: {}", model.provider_name);
        println!("  Vendor: {}", model.vendor_name);
        println!("  Context: {} tokens", model.context_length);
        if let Some(completion) = model.completion_length {
            println!("  Max completion: {} tokens", completion);
        }
        println!("  Capabilities:");
        println!("    - Vision: {}", model.capabilities.supports_vision);
        println!("    - Tools: {}", model.capabilities.supports_tools);
        println!("    - Streaming: {}", model.capabilities.supports_streaming);
        println!(
            "    - Temperature: {}",
            model.capabilities.supports_temperature
        );
        println!(
            "    - System messages: {}",
            model.capabilities.supports_system
        );
        println!(
            "    - Multiple system: {}",
            model.capabilities.supports_multiple_system
        );
        println!(
            "    - Repeating roles: {}",
            model.capabilities.supports_repeating_roles
        );
        println!("  Properties:");
        println!("    - Open source: {}", model.properties.open_source);
        println!("    - GDPR compliant: {}", model.properties.gdpr_compliant);
        if let Some(cutoff) = model.knowledge_cutoff {
            println!("    - Knowledge cutoff: {}", cutoff);
        }
    }

    Ok(())
}
