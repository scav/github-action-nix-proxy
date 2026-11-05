use anyhow::{Context, Result};
use std::env;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub host: Arc<str>,
    pub cache_url: Arc<str>,
    pub auth_token: Arc<str>,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "port: {}, cache_url: {}, token_set: {}",
            self.port,
            self.cache_url,
            !self.auth_token.is_empty()
        )
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        let port = u16::from_str(&env::var("NIXFWD_PORT").unwrap_or("8300".to_string()))?;
        let cache_url: Arc<str> =
            Arc::from(&*env::var("NIXFWD_URL").context("NIXFWD_URL not set")?);
        let host: Arc<str> = Arc::from(&*env::var("NIXFWD_HOST").context("NIXFWD_HOST not set")?);
        let auth_token = env::var("NIXFWD_AUTHORIZATION_TOKEN").context("NIXFWD_TOKEN not set")?;
        let auth_token_name = env::var("NIXFWD_AUTHORIZATION_TOKEN_NAME").unwrap_or_else(|_| {
            info!("NIXFWD_AUTHORIZATION_TOKEN_NAME not set. Using default of Bearer");
            "Bearer".to_string()
        });

        Ok(Config::with_token(
            &auth_token_name,
            port,
            cache_url,
            host,
            auth_token,
        )?)
    }
    pub fn with_token(
        name: &str,
        port: u16,
        cache_url: Arc<str>,
        host: Arc<str>,
        mut auth_token: String,
    ) -> Result<Self> {
        auth_token.insert_str(0, name);
        auth_token.insert_str(name.len(), " ");

        Ok(Self {
            port,
            host,
            cache_url,
            auth_token: Arc::from(&*auth_token),
        })
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::Config;

    #[test]
    fn test_with_token() {
        let cache_url = Arc::from("https://example.org");
        let port = 8300;
        let auth_token = String::from("na");
        let host = Arc::from("example.org");

        let config = Config::with_token("Bearer", port, cache_url, host, auth_token)
            .expect("config should be created");

        assert_eq!(&config.auth_token.to_string(), "Bearer na")
    }
}
