extern crate rustygit;
extern crate rm_rf;
extern crate httparse;
extern crate serde_derive;
extern crate serde_json;

use std::io::prelude::*;
use std::collections::HashMap;
use std::process::Command;
use std::error::Error;
use std::net::TcpListener;

mod read_config;

use serde_derive::Deserialize;

use httparse::Request;

use rustygit::types::BranchName;
use std::str::FromStr;

use read_config::{read_config, Recipe};
use rustygit::types::GitUrl;

#[derive(Hash, PartialEq, Eq)]
struct Target {
    repoository_name: String,
    branch_name: String
}

type Repos = HashMap<Target, rustygit::Repository>;

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

#[derive(Debug, Deserialize)]
struct Repository {
    name: String
}

#[derive(Debug, Deserialize)]
struct Webhook {
    r#ref: String,
    repository: Repository,
}

fn main() -> std::io::Result<(), > {
    let repos = read_config();

    Ok(())
}

// fn main() -> std::io::Result<(), > {
//     let _ = rm_rf::ensure_removed("./repos");
//     // let recipes: Vec<Recipe> = read_config()?.recipes;
//     let listener = TcpListener::bind("127.0.0.1:6000").unwrap();

//     for stream in listener.incoming() {
//         let mut stream = stream.unwrap();
//         let mut buffer = [0; 20_000];
//         let mut headers = [httparse::EMPTY_HEADER; 20];
//         println!("\n\n");
//         let bytes_read = stream.read(&mut buffer).unwrap();

//         let offset = Request::new(&mut headers).parse(&buffer).unwrap().unwrap();

//         // println!("Hopefully body: {}", String::from_utf8_lossy(&buffer[offset..]));
//         let body = String::from_utf8_lossy(&buffer[offset..bytes_read]).to_string();
//         println!("Body: {:?}", &body[body.len() - 1..]);
//         println!("Body length: {:?}", body.len());
//         let maybe_parsed_webhook: Webhook = serde_json::from_str(&body)?;
//         println!("Parsed: {:?}", maybe_parsed_webhook);

//         println!("------------------------------------------------");

//     }
    

//     Ok(())
// }


/*
 * main {
 *   read_config
 *   loop over tcp incoming {
 *     if name and branch in config {
 *       pull updated repo
 *       build it++
 *     } 
 *   }
 * }
 */