extern crate serde_derive;
extern crate toml;

use serde_derive::Deserialize;

use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct HostSettings {
    pub port: u16,
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Recipe {
    pub repository_url: String,
    pub branch: String,
    pub build: Option<Vec<Vec<String>>>,
    pub run: Option<Vec<String>>,
    pub host: Option<HostSettings>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Recipes {
    pub recipes: Vec<Recipe>,
}

pub fn read_recipes() -> Result<Recipes, std::io::Error> {
    let config_raw = fs::read_to_string("./bull.toml")?;

    let recipes = toml::from_str(&config_raw)?;

    return Ok(recipes);
}
