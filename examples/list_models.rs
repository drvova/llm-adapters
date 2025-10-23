use martian_adapters::{AdapterFactory, ModelFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching models from models.dev...");

    AdapterFactory::init_from_modelsdev().await?;

    println!("✓ Models loaded successfully\n");

    let all_models = AdapterFactory::get_supported_models(None).await;
    println!("Total models: {}", all_models.len());

    let vision_models =
        AdapterFactory::get_supported_models(Some(ModelFilter::new().with_vision(true))).await;
    println!("Vision-enabled models: {}", vision_models.len());

    let tool_models =
        AdapterFactory::get_supported_models(Some(ModelFilter::new().with_tools(true))).await;
    println!("Tool-enabled models: {}", tool_models.len());

    let providers = AdapterFactory::list_providers().await;
    println!("\nAvailable providers ({}):", providers.len());
    for provider in providers.iter().take(10) {
        println!("  - {}", provider);
    }

    if let Ok(model) = AdapterFactory::get_model("openai/openai/gpt-4o-mini").await {
        println!("\nExample model: {}", model.name);
        println!("  Provider: {}", model.provider_name);
        println!("  Vendor: {}", model.vendor_name);
        println!("  Context length: {}", model.context_length);
        println!("  Supports vision: {}", model.capabilities.supports_vision);
        println!("  Supports tools: {}", model.capabilities.supports_tools);
        println!("  Cost (prompt): ${:.6} per token", model.cost.prompt);
        println!(
            "  Cost (completion): ${:.6} per token",
            model.cost.completion
        );
    }

    Ok(())
}
