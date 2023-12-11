mod github;
mod git;
mod helpers;

use std::collections::HashMap;
use clap::{Arg, Command, value_parser};
use git::{Git};
use github::{ GitHub };
use crate::helpers::install_zsh_autocompletion;

fn create_arg(name: &'static str, long: &'static str, short: char, help: &'static str) -> Arg {
    Arg::new(name)
        .long(long)
        .short(short)
        .help(help)
        .exclusive(true)
        .value_parser(value_parser!(String))
}

fn create_subcommand(name: &'static str, args: &[Arg], about: &'static str) -> Command {
    let mut command = Command::new(name).about(about);
    for arg in args {
        command = command.arg(arg.clone());
    }
    command
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ref_arg = create_arg("ref", "ref", 'r', "The name of the ref tree");
    let owner_arg = create_arg("owner", "owner", 'o', "The owner of the repository where the action is located.");
    let repo_arg = create_arg("repo", "repo", 'p', "The name of the repository where the action is located.");
    let token_arg = create_arg("token", "token", 't', "The token used for authentication. If not provided, the GAR_TOKEN environment variable will be used.");
    let inputs_arg = create_arg("inputs", "inputs", 'i', "The name of the event that triggers the action.");

    let common_args = vec![ref_arg.clone(), owner_arg.clone(), repo_arg.clone(), token_arg.clone()];

    let mut gar_command = Command::new("gar").bin_name("gar");
    for arg in &common_args {
        gar_command = gar_command.arg(arg.clone());
    }
    gar_command = gar_command.arg(inputs_arg.clone());

    let history_command = create_subcommand("history", &common_args, "Shows the history of workflow runs");
    let details_command = create_subcommand("details", &common_args, "Shows the details of workflow run");
    let autocomplete_command = create_subcommand("autocomplete", &[], "add autocomplete to zsh");

    gar_command = gar_command
        .subcommand(history_command)
        .subcommand(details_command)
        .subcommand(autocomplete_command);

    let matches = gar_command.subcommand_required(false).get_matches();

    let token = matches.get_one::<String>("token").map(|s| s.to_owned()).unwrap_or_else(|| Git::get_token().unwrap());
    let owner = matches.get_one::<String>("owner").map(|s| s.to_owned()).unwrap_or_else(|| Git::get_git_owner().unwrap());
    let repo = matches.get_one::<String>("repo").map(|s| s.to_owned()).unwrap_or_else(|| Git::get_git_repo().unwrap());
    let ref_name = matches.get_one::<String>("ref").map(|s| s.to_owned()).unwrap_or_else(|| Git::get_git_tree_name().unwrap_or("main".parse().unwrap()));


    let github = GitHub::new(token, owner, repo);
    match matches.subcommand() {
        Some(("history", _)) => {
            // Запустите функцию, которая показывает историю запусков
            github.show_history().await?;
        }
        Some(("details", _)) => {
            github.show_details().await?;
        }
        Some(("autocomplete", _)) => {
            install_zsh_autocompletion()?;
        }
        _ => {
            let inputs = matches.get_one::<String>("inputs").map(|s| s.to_owned()).unwrap_or_else(String::new);

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

            github.run_workflow(&ref_name, inputs_collect).await?;
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
}
