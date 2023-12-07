mod github;
mod git;
mod cli;

use std::{env};
use std::collections::HashMap;
use std::fs;
use dialoguer::{Input, theme::ColorfulTheme};
use clap::{Arg, Command, value_parser};
use git::{get_git_owner, get_git_repo, get_git_tree_name};
use github::{run_workflow, show_history, show_details};

fn get_token() -> Result<String, Box<dyn std::error::Error>> {
    let token = match env::var("GAR_TOKEN") {
        Ok(val) => val,
        Err(_) => {
            if fs::metadata(".github_token").is_ok() {
                let token_content = fs::read_to_string(".github_token")?;
                let trimmed_token = token_content.trim();
                String::from(trimmed_token)
            } else {
                Input::<String>::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter github token")
                    .interact()?
            }
        }
    };
    Ok(token)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("gar")
        .bin_name("gar")
        .arg(
            Arg::new("ref")
                .long("ref")
                .short('r')
                .help("The name of the ref tree")
                .exclusive(true)
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("owner")
                .long("owner")
                .short('o')
                .help("The owner of the repository where the action is located.")
                .exclusive(true)
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("repo")
                .long("repo")
                .short('p')
                .help("The name of the repository where the action is located.")
                .exclusive(true)
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("token")
                .long("token")
                .short('t')
                .help("The token used for authentication. If not provided, the GAR_TOKEN environment variable will be used.")
                .exclusive(true)
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("inputs")
                .long("inputs")
                .short('i')
                .help("The name of the event that triggers the action.")
                .exclusive(true)
                .value_parser(value_parser!(String)),
        )
        .subcommand_required(false)
        .subcommand(
            Command::new("history")
                .about("Shows the history of workflow runs"),
        ).subcommand(
            Command::new("details")
                .about("Shows the details of workflow run"),
        ).get_matches();

    let token = match matches.get_one::<String>("token") {
        Some(owner_name) => owner_name.to_string(),
        None => get_token().unwrap(),
    };

    let owner = match matches.get_one::<String>("owner") {
        Some(owner_name) => owner_name.to_string(),
        None => get_git_owner()?,
    };

    let repo = match matches.get_one::<String>("repo") {
        Some(repo_name) => repo_name.to_string(),
        None => get_git_repo()?,
    };

    let ref_name = match matches.get_one::<String>("ref") {
        Some(ref_name) => ref_name.to_string(),
        None => get_git_tree_name().unwrap_or("main".parse().unwrap()),
    };

    let inputs = match matches.get_one::<String>("inputs") {
        Some(inputs) => inputs.to_string(),
        None => "".to_string(),
    };

    let inputs_collect: HashMap<_, _> = inputs
        .split('&')
        .filter_map(|pair| {
            let mut split = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (split.next(), split.next()) {
                Some((key, value))
            } else {
                None
            }
        })
        .collect();

    match matches.subcommand() {
        Some(("history", _)) => {
            // Запустите функцию, которая показывает историю запусков
            show_history(&token, &owner, &repo).await?;
        }
        Some(("details", _)) => {
            show_details(&token, &owner, &repo).await?;
        }
        _ => {
            run_workflow(&token, &owner, &repo, &ref_name, inputs_collect).await?;
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