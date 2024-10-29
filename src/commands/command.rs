use clap::{Arg, Command as CommandClap, value_parser};
use crate::git::Git;


pub struct Command {
    pub token: String,
    pub owner: String,
    pub repo: String,
    pub ref_name: String,
}

impl Command {
    pub fn new(matches: &clap::ArgMatches) -> Self {
        let token = matches.get_one::<String>("token").map(|s| s.to_owned()).unwrap_or_else(|| Git::get_token().unwrap());
        let owner = matches.get_one::<String>("owner").map(|s| s.to_owned()).unwrap_or_else(|| Git::get_git_owner().unwrap());
        let repo = matches.get_one::<String>("repo").map(|s| s.to_owned()).unwrap_or_else(|| Git::get_git_repo().unwrap());
        let ref_name = matches.get_one::<String>("ref").map(|s| s.to_owned()).unwrap_or_else(|| Git::get_git_tree_name().unwrap_or("main".parse().unwrap()));

        Command {
            token,
            owner,
            repo,
            ref_name,
        }
    }

    pub fn create_arg(name: &'static str, long: &'static str, short: char, help: &'static str) -> Arg {
        Arg::new(name)
            .long(long)
            .short(short)
            .help(help)
            .exclusive(true)
            .value_parser(value_parser!(String))
    }

    pub fn create_subcommand(name: &'static str, args: &[Arg], about: &'static str) -> CommandClap {
        let mut command = CommandClap::new(name).about(about);
        for arg in args {
            command = command.arg(arg.clone());
        }
        command
    }

    pub fn create_command() -> CommandClap {
        let ref_arg = Self::create_arg("ref", "ref", 'r', "The name of the ref tree");
        let owner_arg = Self::create_arg("owner", "owner", 'o', "The owner of the repository where the action is located.");
        let repo_arg = Self::create_arg("repo", "repo", 'p', "The name of the repository where the action is located.");
        let token_arg = Self::create_arg("token", "token", 't', "The token used for authentication. If not provided, the GAR_TOKEN environment variable will be used.");
        let inputs_arg = Self::create_arg("inputs", "inputs", 'i', "The name of the event that triggers the action.");

        let common_args = vec![ref_arg.clone(), owner_arg.clone(), repo_arg.clone(), token_arg.clone()];

        let mut gar_command = CommandClap::new("gar").bin_name("gar");
        for arg in &common_args {
            gar_command = gar_command.arg(arg.clone());
        }
        gar_command = gar_command.arg(inputs_arg.clone());

        let history_command = Self::create_subcommand("history", &common_args, "Shows the history of workflow runs");
        let details_command = Self::create_subcommand("details", &common_args, "Shows the details of workflow run");
        let autocomplete_command = Self::create_subcommand("autocomplete", &[], "add autocomplete to zsh");

        gar_command = gar_command
            .subcommand(history_command)
            .subcommand(details_command)
            .subcommand(autocomplete_command);

        gar_command
    }
}