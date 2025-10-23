use crate::config::EnvConfig;
use crate::error::Result;
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Result<Self> {
        let timeout = EnvConfig::get_http_timeout();
        let connect_timeout = EnvConfig::get_http_connect_timeout();

        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout))
            .connect_timeout(Duration::from_secs(connect_timeout))
            .pool_max_idle_per_host(EnvConfig::get_max_keepalive_connections())
            .build()?;

        Ok(Self { client })
    }

    pub fn with_timeout(timeout_secs: u64) -> Result<Self> {
        let connect_timeout = EnvConfig::get_http_connect_timeout();

        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(timeout_secs))
            .connect_timeout(Duration::from_secs(connect_timeout))
            .pool_max_idle_per_host(EnvConfig::get_max_keepalive_connections())
            .build()?;

        Ok(Self { client })
    }

    pub fn inner(&self) -> &Client {
        &self.client
    }
}
