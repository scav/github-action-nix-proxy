use anyhow::{Context, Result};
use std::env;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub cache_url: Arc<str>,
}

impl Config {
    pub fn new() -> Result<Self> {
        Ok(Self {
            port: u16::from_str(&env::var("NIXFWD_PORT").unwrap_or("8300".to_string()))?,
            cache_url: Arc::from(
                &*env::var("NIXFWD_URL").context("Could not create Uri from NIXFWD_URL")?,
            ),
        })
    }
}
