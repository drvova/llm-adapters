use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Provider not supported: {0}")]
    ProviderNotSupported(String),

    #[error("API key not found for provider: {0}")]
    ApiKeyNotFound(String),

    #[error("Model does not support {feature}: {model}")]
    UnsupportedFeature { model: String, feature: String },

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, AdapterError>;
