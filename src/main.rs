use std::env;
use std::fs;
use std::process::Command;
use dialoguer::{Input, Confirm, Select, theme::ColorfulTheme};
use regex::Regex;
use serde_json::json;
use reqwest::Client;
use git2::Repository;
use colored::*;

struct Workflow {
    id: String,
    name: String,
    html_url: String,
}

async fn github_request(url: &str, token: &str, method: &str, data: Option<serde_json::Value>) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = Client::new();

    let response = match method {
        "POST" => client.post(url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("token {}", token))
            .json(&data.unwrap())
            .send()
            .await?,
        _ => client.get(url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("token {}", token))
            .send()
            .await?,
    };

    let data: serde_json::Value = response.json().await?;
    Ok(data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("CTIM_TOKEN").unwrap_or_else(|_| {
        if fs::metadata(".github_token").is_ok() {
            fs::read_to_string(".github_token").unwrap()
        } else {
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter github token")
                .interact()
                .unwrap()
        }
    });

    let owner = get_git_owner()?;
    let repo = get_git_repo()?;
    let ref_name = get_git_tree_name()?;

    let url = format!("https://api.github.com/repos/{}/actions/workflows", owner);
    let workflows_data = github_request(&url, &token, "GET", None).await?;
    let workflows: Vec<Workflow> = serde_json::from_value(workflows_data["workflows"].clone())?;

    let workflow_names: Vec<&str> = workflows.iter().map(|wf| wf.name.as_str()).collect();
    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a workflow:")
        .items(&workflow_names)
        .default(0)
        .interact()
        .unwrap();

    let workflow = &workflows[selected];
    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(&format!("Run \"{}\"({}) action in \"{}\" tree?", workflow.name, workflow.html_url, ref_name))
        .interact()
        .unwrap();

    if confirm {
        let url = format!("https://api.github.com/repos/{}/actions/workflows/{}/dispatches", owner, workflow.id);
        let _ = github_request(&url, &token, "POST", Some(json!({ "ref": ref_name }))).await?;
        println!("GitHub action successfully triggered.\nActions: https://github.com/{}/actions\nTree: https://github.com/{}/tree/{}", owner, repo, ref_name);
    } else {
        println!("{}", "Cancel".red());
    }

    Ok(())
}

fn get_git_owner() -> Result<String, git2::Error> {
    let repo = Repository::open(".")?;
    let origin = repo.find_remote("origin")?;
    let url = origin.url().unwrap();
    let re = Regex::new(r"github\.com[/:](.*?)/").unwrap();
    let caps = re.captures(url).unwrap();
    Ok(caps.get(1).map_or("", |m| m.as_str()).to_string())
}

fn get_git_repo() -> Result<String, git2::Error> {
    let repo = Repository::open(".")?;
    let origin = repo.find_remote("origin")?;
    let url = origin.url().unwrap();
    let re = Regex::new(r"github\.com[/:].*?/(.*?)(\.git)?$").unwrap();
    let caps = re.captures(url).unwrap();
    Ok(caps.get(1).map_or("", |m| m.as_str()).to_string())
}

fn get_git_tree_name() -> Result<String, git2::Error> {
    let repo = Repository::open(".")?;
    let head = repo.head()?;
    let name = head.shorthand().unwrap();
    Ok(name.to_string())
}