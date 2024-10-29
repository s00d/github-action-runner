use crate::github::GitHub;
use super::command::Command;
use crate::helpers::{unzip_and_concatenate};

pub struct DetailsCommand {
    command: Command,
}

impl DetailsCommand {
    pub fn new(command: Command) -> Self {
        DetailsCommand { command }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let github = GitHub::new(self.command.token.clone(), self.command.owner.clone(), self.command.repo.clone());
        let workflow = github.select_workflow().await?;
        let run = github.select_run(workflow.id).await?;

        println!("ID: {}", run.id);
        println!("Name: {}", run.name);
        println!("Display Title: {}", run.display_title);
        println!("URL: {}", run.html_url);
        println!("Status: {}", run.status);
        println!("Conclusion: {}", run.conclusion.unwrap_or_else(|| "N/A".to_string()));
        println!("Branch: {}", run.head_branch);
        println!("Created At: {}", run.created_at);
        println!("Updated At: {}", run.updated_at);

        let logs_data = github.github_request_bytes(run.logs_url.as_str(), "GET", None, Some("application/vnd.github+json")).await?;
        let logs = unzip_and_concatenate(logs_data.clone());
        println!("Logs: \n{}", logs.unwrap());

        // https://api.github.com/repos/s00d/github-action-runner/actions/runs/7090586915/logs
        // https://api.github.com/repos/s00d/github-action-runner/actions/runs/7090586915/logs

        Ok(())
    }
}