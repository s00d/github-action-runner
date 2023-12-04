mod github;
mod git;
mod cli;

use std::{env};
use std::fs;
use dialoguer::{Input, theme::ColorfulTheme};
use clap::{Command};
use git::{get_git_owner, get_git_repo, get_git_tree_name};
use github::{run_workflow, show_history};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = Command::new("gar")
        .bin_name("gar")
        .subcommand_required(false)
        .subcommand(
            Command::new("history")
                .about("Shows the history of workflow runs"),
        );

    let matches = cmd.get_matches();

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

    match matches.subcommand() {
        Some(("run", _)) => {
            // Запустите функцию, которая выполняет текущую функциональность

        }
        Some(("history", _)) => {
            // Запустите функцию, которая показывает историю запусков
            show_history(&token, &owner, &repo).await?;
        }
        _ => {
            run_workflow(&token, &owner, &repo, &ref_name).await?;
        },
    };

    Ok(())
}


#[cfg(test)]
mod tests {
    use regex::Regex;
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