use anyhow::{Context, Result};
use std::env;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub cache_url: Arc<str>,
    pub token: Arc<str>,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "port: {}, cache_url: {}, token: {}",
            self.port,
            self.cache_url,
            !self.token.is_empty()
        )
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        Ok(Self {
            port: u16::from_str(&env::var("NIXFWD_PORT").unwrap_or("8300".to_string()))?,
            cache_url: Arc::from(
                &*env::var("NIXFWD_URL").context("Could not create Uri from NIXFWD_URL")?,
            ),
            token: Arc::from(
                env::var("NIXFWD_TOKEN").context("Could not read token from NIXFWD_TOKEN")?,
            ),
        })
    }
}
