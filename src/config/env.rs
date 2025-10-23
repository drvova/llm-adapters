use std::env;

pub struct EnvConfig;

impl EnvConfig {
    pub fn get_api_key(provider: &str) -> Option<String> {
        let key_name = format!("{}_API_KEY", provider.to_uppercase().replace('-', "_"));
        env::var(&key_name).ok()
    }

    pub fn get_override_base_url() -> Option<String> {
        env::var("_ADAPTERS_OVERRIDE_ALL_BASE_URLS_").ok()
    }

    pub fn get_max_connections() -> usize {
        env::var("ADAPTERS_MAX_CONNECTIONS_PER_PROCESS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000)
    }

    pub fn get_max_keepalive_connections() -> usize {
        env::var("ADAPTERS_MAX_KEEPALIVE_CONNECTIONS_PER_PROCESS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100)
    }

    pub fn get_http_timeout() -> u64 {
        env::var("ADAPTERS_HTTP_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(600)
    }

    pub fn get_http_connect_timeout() -> u64 {
        env::var("ADAPTERS_HTTP_CONNECT_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5)
    }
}
