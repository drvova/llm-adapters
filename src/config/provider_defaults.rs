use crate::error::Result;
use crate::models::ModelCapabilities;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderDefaults {
    #[serde(flatten)]
    pub capabilities: ModelCapabilities,
    #[serde(skip)]
    pub base_url: Option<String>,
}

static PROVIDER_DEFAULTS: Lazy<HashMap<String, ProviderDefaults>> = Lazy::new(|| {
    let config_str = include_str!("../../config/provider_defaults.toml");
    toml::from_str(config_str).expect("Failed to parse provider_defaults.toml")
});

impl ProviderDefaults {
    pub fn for_provider(provider_id: &str) -> ProviderDefaults {
        PROVIDER_DEFAULTS
            .get(provider_id)
            .cloned()
            .unwrap_or_else(|| ProviderDefaults {
                capabilities: ModelCapabilities::default(),
                base_url: None,
            })
    }

    pub fn get_all() -> Result<HashMap<String, ProviderDefaults>> {
        Ok(PROVIDER_DEFAULTS.clone())
    }
}
