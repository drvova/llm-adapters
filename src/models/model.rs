use crate::models::cost::Cost;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    #[serde(default = "default_true")]
    pub supports_user: bool,
    #[serde(default = "default_true")]
    pub supports_repeating_roles: bool,
    #[serde(default = "default_true")]
    pub supports_streaming: bool,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default)]
    pub supports_tools: bool,
    #[serde(default = "default_true")]
    pub supports_n: bool,
    #[serde(default = "default_true")]
    pub supports_system: bool,
    #[serde(default = "default_true")]
    pub supports_multiple_system: bool,
    #[serde(default = "default_true")]
    pub supports_empty_content: bool,
    #[serde(default)]
    pub supports_tool_choice: bool,
    #[serde(default)]
    pub supports_tool_choice_required: bool,
    #[serde(default = "default_true")]
    pub supports_json_output: bool,
    #[serde(default = "default_true")]
    pub supports_json_content: bool,
    #[serde(default = "default_true")]
    pub supports_last_assistant: bool,
    #[serde(default = "default_true")]
    pub supports_first_assistant: bool,
    #[serde(default = "default_true")]
    pub supports_temperature: bool,
    #[serde(default = "default_true")]
    pub supports_only_system: bool,
    #[serde(default = "default_true")]
    pub supports_only_assistant: bool,
}

fn default_true() -> bool {
    true
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
    #[serde(default)]
    pub open_source: bool,
    #[serde(default)]
    pub chinese: bool,
    #[serde(default)]
    pub gdpr_compliant: bool,
    #[serde(default)]
    pub is_nsfw: bool,
}

impl Default for ModelProperties {
    fn default() -> Self {
        Self {
            open_source: false,
            chinese: false,
            gdpr_compliant: false,
            is_nsfw: false,
        }
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub knowledge_cutoff: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}

impl Model {
    pub fn get_path(&self) -> String {
        format!("{}/{}/{}", self.provider_name, self.vendor_name, self.name)
    }
}
