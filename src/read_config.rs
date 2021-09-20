extern crate serde_derive;
extern crate toml;

use serde_derive::Deserialize;

use std::fs;

#[derive(Debug, Deserialize)]
pub struct Recipe {
    pub repository_url: String,
    pub branch: String,
    pub build: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub recipes: Vec<Recipe>,
}

pub fn read_config() -> Result<Config, std::io::Error> {
  let config_raw = fs::read_to_string("./bull.toml")?;

  let config: Config = toml::from_str(&config_raw)?;

  return Ok(config);
}