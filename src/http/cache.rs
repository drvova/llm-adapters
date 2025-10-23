use crate::http::HttpClient;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

type CacheKey = (String, String);

static CLIENT_CACHE: Lazy<DashMap<CacheKey, HttpClient>> = Lazy::new(DashMap::new);

pub struct ClientCache;

impl ClientCache {
    pub fn get_or_create(base_url: &str, api_key: &str) -> HttpClient {
        let key = Self::make_key(base_url, api_key);

        CLIENT_CACHE
            .entry(key)
            .or_insert_with(|| HttpClient::new().expect("Failed to create HTTP client"))
            .clone()
    }

    fn make_key(base_url: &str, api_key: &str) -> CacheKey {
        let mut hasher = DefaultHasher::new();
        api_key.hash(&mut hasher);
        let api_key_hash = format!("{:x}", hasher.finish());

        (base_url.to_string(), api_key_hash)
    }

    pub fn clear() {
        CLIENT_CACHE.clear();
    }
}
