use std::env;
use std::fs;
use dialoguer::{Input, Confirm, Select, theme::ColorfulTheme};
use regex::Regex;
use serde_json::json;
use reqwest::Client;
use git2::Repository;
use colored::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Workflow {
    id: u64,
    name: String,
    html_url: String,
}

async fn github_request(url: &str, token: &str, method: &str, data: Option<serde_json::Value>) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let proxy = reqwest::Proxy::all("http://127.0.0.1:4034")?; // Замените на адрес вашего прокси
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .proxy(proxy)
        .build()?;

    let response = match method {
        "POST" => client.post(url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("token {}", token))
            .header("User-Agent", "GAR")
            .json(&data.unwrap())
            .send()
            .await?,
        _ => client.get(url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("token {}", token))
            .header("User-Agent", "GAR")
            .send()
            .await?,
    };

    let response_text = response.text().await?;
    println!("Response text: {}", response_text);

    if response_text.trim().is_empty() {
        return Ok(serde_json::Value::Null);
    }

    let data: serde_json::Value = serde_json::from_str(&response_text)?;

    println!("{}", data);
    Ok(data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("GAR_TOKEN").unwrap_or_else(|_| {
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

    let url = format!("https://api.github.com/repos/{}/{}/actions/workflows", owner, repo);
    let workflows_data = github_request(&url, &token, "GET", None).await?;
    let workflows: Vec<Workflow> = serde_json::from_value(workflows_data["workflows"].clone())?;

    let workflow_names: Vec<String> = workflows.iter().map(|wf| {
        let mut name = wf.name.clone();
        if wf.name.to_lowercase().contains("prod") {
            name = format!(" !!! {} ", name).red().to_string();
        }
        if wf.name.to_lowercase().contains("test") {
            name = format!(" {} ", name).blue().to_string();
        }
        name
    }).collect();

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
        let url = format!("https://api.github.com/repos/{}/{}/actions/workflows/{}/dispatches", owner, repo, workflow.id);
        let _ = github_request(&url, &token, "POST", Some(json!({ "ref": ref_name }))).await?;
        println!("GitHub action successfully triggered.\nActions: https://github.com/{}/{}/actions\nTree: https://github.com/{}/tree/{}", owner, repo, repo, ref_name);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_git_owner() {
        let url = "https://github.com/owner/repo.git";
        let re = Regex::new(r"github\.com[/:](.*?)/").unwrap();
        let caps = re.captures(url).unwrap();
        let owner = caps.get(1).map_or("", |m| m.as_str()).to_string();
        assert_eq!(owner, "owner");
    }

    #[test]
    fn test_get_git_repo() {
        let url = "https://github.com/owner/repo.git";
        let re = Regex::new(r"github\.com[/:].*?/(.*?)(\.git)?$").unwrap();
        let caps = re.captures(url).unwrap();
        let repo = caps.get(1).map_or("", |m| m.as_str()).to_string();
        assert_eq!(repo, "repo");
    }

    // Assuming the repository is on the "main" branch
    #[test]
    fn test_get_git_tree_name() {
        let tree_name = get_git_tree_name().unwrap();
        assert_eq!(tree_name, "main");
    }
}