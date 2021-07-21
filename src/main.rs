extern crate rustygit;
extern crate rm_rf;

use std::process::Command;
use std::error::Error;

mod read_config;

use rustygit::types::BranchName;
use std::str::FromStr;

use read_config::{read_config, Recipe};
use rustygit::types::GitUrl;

// Goes through the list of recipes and clones all the repos
fn initialize(recipes: &Vec<Recipe>) -> Result<(), Box<dyn Error>> {
    for recipe in recipes {
        let name = recipe.repository_url.split('/').last().unwrap();
        let path = format!("{}/{}", "repos", name);

        let repo = rustygit::Repository::clone(GitUrl::from_str(&recipe.repository_url)?, path)?;

        // Apparently not necessary
        repo.fetch_remote(&recipe.repository_url)?;

        let _ = repo.switch_branch(&BranchName::from_str(&recipe.branch).unwrap());
        println!("{:?}", repo.list_branches());
        
        let mut context_dir = std::env::current_dir()?;
        context_dir.push("repos");
        context_dir.push(name);

        println!("Context directory: {:?}", context_dir);
        
        let status = Command::new("cargo").arg("build").current_dir(context_dir).status()?;

        println!("{:?}", status)
    }
    return Ok(());
}


fn main() -> std::io::Result<(), > {
    let _ = rm_rf::ensure_removed("./repos");

    let recipes: Vec<Recipe> = read_config()?.recipes;
    initialize(&recipes).unwrap();

    Ok(())
}
