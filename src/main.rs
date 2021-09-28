use std::collections::HashMap;
use std::error::Error;
use std::io::prelude::*;
use std::net::TcpListener;
use std::process::Command;

mod read_config;

use serde_derive::Deserialize;

use httparse::Request;

use rustygit::types::BranchName;
use std::str::FromStr;

use read_config::{read_recipes, Recipe};
use rustygit::types::GitUrl;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct Target {
    repository_name: String,
    branch_name: String,
}

fn get_context_dir(name: &str) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let mut dir = std::env::current_dir()?;
    dir.push("repos");
    dir.push(name);

    Ok(dir)
}

struct Repository {
    recipe: Recipe,
    repository: rustygit::Repository
}

impl std::fmt::Debug for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result  {
        f.debug_struct("Repository").field("Branch name", &self.recipe.branch).finish()
    }
}

type Repos = HashMap<Target, Repository>;

// Goes through the list of recipes and clones all the repos
fn initialize_repos(recipes: &Vec<Recipe>) -> Result<Repos, Box<dyn Error>> {
    let _ = rm_rf::ensure_removed("./repos");

    let mut repos: Repos = HashMap::new();
    for recipe in recipes {
        let name = recipe.repository_url.split('/').last().unwrap();
        let name = match name.strip_suffix(".git") {
            Some(s) => s,
            None=> name
        };
        let path = format!("{}/{}", "repos", name);
        let repo = rustygit::Repository::clone(GitUrl::from_str(&recipe.repository_url)?, path)?;
        let _ = repo.switch_branch(&BranchName::from_str(&recipe.branch).unwrap());
        let target = Target {
            repository_name: name.to_string(),
            branch_name: recipe.branch.clone(),
        };
        repos.insert(target, Repository { recipe: recipe.clone(), repository: repo});
    }

    Ok(repos)
}

#[derive(Debug, Deserialize)]
struct WHRepository {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Webhook {
    r#ref: String,
    repository: WHRepository,
}

fn initialize() -> Result<Repos, Box<dyn Error>> {
    let recipes = read_recipes()?.recipes;
    let repositories = initialize_repos(&recipes)?;

    Ok(repositories)
}
fn serve_static_files(port: u16, path: String) -> Result<(), Box<dyn Error>> {
    // let listener = std::net::TcpListener::bind(format!("localhost:{}", port))?;
    
    // for stream in listener.incoming() {
    //     let mut stream = stream.unwrap();

    //     let response = "HTTP/1.1 200 OK\r\n\r\n";

    //     stream.write(response.as_bytes())?;
    //     stream.flush()?;
    // }

    let mut mount = mount::Mount::new();
    // let handler = staticfile::Static::new(std::path::Path::new(&path));
    let handler = staticfile::Static::new(std::path::Path::new("~/Documents"));
    mount.mount("", handler);
    iron::Iron::new(mount).http(("127.0.0.1", port)).expect("Failed to serve");
    
    Ok(())
}

// See this page for an alternative way of running complex commands on *NIX and Windows
// https://doc.rust-lang.org/std/process/struct.Command.html
fn handle_target(target: Target, repos: &Repos) -> Result<(), Box<dyn Error>> {
    if  let Some(repo) = repos.get(&target) {
        repo.repository.switch_branch(&BranchName::from_str(&target.branch_name)?)?;
        let context_dir = get_context_dir(&target.repository_name)?;

        // First run all the build steps, if any
        if let Some(commands) = &repo.recipe.build {
            for command in commands {
                let mut cmd = Command::new(command[0].clone());
                let cmd = cmd.args(&command[1..]).current_dir(&context_dir);
                let status = cmd.status();
                println!("A build command exited with status: {:?}", status)
            }
        }
        
        // Determine if we are running in "Host" or in "Run" mode
        if let Some(host_settings) = &repo.recipe.host {
            let static_files_path = format!("{}/{}", context_dir.to_str().unwrap(), host_settings.path);
            println!("{}", static_files_path);
            let port = host_settings.port;
            let _thread_handle = std::thread::spawn(move || {
                let _ = serve_static_files(port, static_files_path);
            });
        }
        else if let Some(_run_settings) = &repo.recipe.run {
            unimplemented!();
        }
        else {
            unreachable!("Targets must provide either a host or a run config")
        }
    }
    else {
        println!("No match found for target {:?}", target)
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Initializing repos");
    let repos = initialize()?;
    let listener = listen_for_webhooks()?;

    println!("Listening on port 6000");
    for stream in listener.incoming() {
        let target = parse_incoming_webhook(stream.unwrap())?;
        if let Some(target) = target {
            handle_target(target, &repos)?;
        }
    }

    Ok(())
}

#[derive(Debug)]
struct ParseRefError {
    msg: String,
}

impl std::fmt::Display for ParseRefError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Failed to extract branch name from ref")
    }
}

impl Error for ParseRefError {}

fn get_branch_name_from_ref(gitref: String) -> Result<String, Box<dyn Error>> {
    let re = regex::Regex::new(r"refs/heads/(?P<branch_name>.*)")?;
    if let Some(caps) = re.captures(&gitref) {
        if let Some(branch_name) = caps.name("branch_name") {
            let res = branch_name.as_str().into();
            println!("Branch: {:?}", res);
            return Ok(res);
        }
        else {
            return Err(Box::new(ParseRefError { msg: String::from("Oh no") } )); 
        }
    }
    else {
        return Err(Box::new(ParseRefError { msg: String::from("Oh no") } ));
    }
}

fn parse_incoming_webhook(
    mut stream: std::net::TcpStream,
) -> Result<Option<Target>, Box<dyn Error>> {
    println!("------------------------------------------------");
    println!("--------------- Incoming Webhook ---------------");
    println!("------------------------------------------------\n");

    let mut buffer = [0; 20_000];
    let mut headers = [httparse::EMPTY_HEADER; 20];

    let nb_bytes_read = stream.read(&mut buffer).unwrap();

    let res =
        if let httparse::Status::Complete(offset) = Request::new(&mut headers).parse(&buffer)? {
            let body = String::from_utf8_lossy(&buffer[offset..nb_bytes_read]).to_string();
            // if an error occurs around here, the webhook may not have the right content type (application/json)
            let parsed_webhook: Webhook = serde_json::from_str(&body)?;
            println!(
                "Parsed a webhook for: {:?} at ref {:?}",
                parsed_webhook.repository.name, parsed_webhook.r#ref
            );
            Ok(Some(Target {
                repository_name: parsed_webhook.repository.name,
                branch_name: get_branch_name_from_ref(parsed_webhook.r#ref)?,
            }))
        } else {
            println!("Failed to parse the incoming request as a GitHub webhook.");
            Ok(None)
        };

    println!("\n................................................\n\n");
    res
}

fn listen_for_webhooks() -> std::io::Result<std::net::TcpListener> {
    let listener = TcpListener::bind("127.0.0.1:6000").unwrap();
    Ok(listener)
}
