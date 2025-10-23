use crate::config::{ProviderDefaults, VendorMappings};
use crate::error::{AdapterError, Result};
use crate::models::{Cost, Model, ModelProperties, ModelsDevResponse};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct AdapterFactory {
    models: HashMap<String, Model>,
}

static FACTORY: Lazy<RwLock<AdapterFactory>> = Lazy::new(|| RwLock::new(AdapterFactory::new()));

impl Default for AdapterFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl AdapterFactory {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub async fn init_from_modelsdev() -> Result<()> {
        let response = Self::fetch_modelsdev_api().await?;
        let mut factory = FACTORY.write().await;
        factory.populate_from_modelsdev(response)?;
        Ok(())
    }

    async fn fetch_modelsdev_api() -> Result<ModelsDevResponse> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://models.dev/api.json")
            .send()
            .await?
            .json::<ModelsDevResponse>()
            .await?;
        Ok(response)
    }

    fn populate_from_modelsdev(&mut self, response: ModelsDevResponse) -> Result<()> {
        for (provider_id, provider) in response.providers {
            for (model_id, model_info) in provider.models {
                let model = self.convert_modelsdev_model(&provider_id, &model_id, &model_info)?;

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
        model_info: &crate::models::ModelInfo,
    ) -> Result<Model> {
        let vendor_name = VendorMappings::extract_vendor(model_id, provider_id);

        let defaults = ProviderDefaults::for_provider(provider_id);

        let mut capabilities = defaults.capabilities.clone();
        capabilities.supports_vision = model_info.modalities.input.contains(&"image".to_string());
        capabilities.supports_tools = model_info.tool_call;
        capabilities.supports_temperature = model_info.temperature;

        let cost = if let Some(cost_info) = &model_info.cost {
            Cost::from_modelsdev(cost_info.input, cost_info.output)
        } else {
            Cost::default()
        };

        Ok(Model {
            name: model_id.to_string(),
            vendor_name,
            provider_name: provider_id.to_string(),
            cost,
            context_length: model_info.limit.context,
            completion_length: Some(model_info.limit.output),
            capabilities,
            properties: ModelProperties {
                open_source: model_info.open_weights,
                chinese: VendorMappings::is_chinese_model(model_id, provider_id),
                gdpr_compliant: VendorMappings::is_gdpr_compliant(provider_id),
                is_nsfw: false,
            },
            knowledge_cutoff: model_info.knowledge.clone(),
            release_date: model_info.release_date.clone(),
            last_updated: model_info.last_updated.clone(),
        })
    }

    pub async fn get_model(model_path: &str) -> Result<Model> {
        let factory = FACTORY.read().await;
        factory
            .models
            .get(model_path)
            .cloned()
            .ok_or_else(|| AdapterError::ModelNotFound(model_path.to_string()))
    }

    pub async fn get_supported_models(filter: Option<ModelFilter>) -> Vec<Model> {
        let factory = FACTORY.read().await;
        factory
            .models
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

    pub async fn list_providers() -> Vec<String> {
        let factory = FACTORY.read().await;
        let mut providers: Vec<String> = factory
            .models
            .values()
            .map(|m| m.provider_name.clone())
            .collect();
        providers.sort();
        providers.dedup();
        providers
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_streaming(mut self, value: bool) -> Self {
        self.supports_streaming = Some(value);
        self
    }

    pub fn with_vision(mut self, value: bool) -> Self {
        self.supports_vision = Some(value);
        self
    }

    pub fn with_tools(mut self, value: bool) -> Self {
        self.supports_tools = Some(value);
        self
    }

    pub fn with_provider(mut self, provider: String) -> Self {
        self.provider = Some(provider);
        self
    }

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
