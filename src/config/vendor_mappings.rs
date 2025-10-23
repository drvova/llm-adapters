use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct VendorMappingsConfig {
    pub patterns: HashMap<String, String>,
    pub provider_defaults: HashMap<String, String>,
}

static VENDOR_MAPPINGS: Lazy<VendorMappingsConfig> = Lazy::new(|| {
    let config_str = include_str!("../../config/vendor_mappings.toml");
    toml::from_str(config_str).expect("Failed to parse vendor_mappings.toml")
});

pub struct VendorMappings;

impl VendorMappings {
    pub fn extract_vendor(model_id: &str, provider_id: &str) -> String {
        for (pattern, vendor) in &VENDOR_MAPPINGS.patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(model_id) {
                    return vendor.clone();
                }
            }
        }

        VENDOR_MAPPINGS
            .provider_defaults
            .get(provider_id)
            .cloned()
            .unwrap_or_else(|| provider_id.to_string())
    }

    pub fn is_chinese_model(model_id: &str, provider_id: &str) -> bool {
        provider_id.contains("china")
            || provider_id.contains("alibaba")
            || provider_id.contains("moonshot")
            || model_id.contains("qwen")
    }

    pub fn is_gdpr_compliant(provider_id: &str) -> bool {
        matches!(provider_id, "openai" | "azure" | "anthropic")
    }
}
