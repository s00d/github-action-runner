use std::collections::HashMap;
use std::sync::Arc;
use colored::Colorize;
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::json;
use tokio::sync::Mutex;
use crate::github::GitHub;
use crate::helpers::{beep, update_progress_bar};
use super::command::Command;

pub struct RunWorkflowCommand {
    command: Command,
    inputs: HashMap<String, String>,
}

impl RunWorkflowCommand {
    pub fn new(command: Command, inputs: String) -> Self {
        let inputs_collect: HashMap<_, _> = inputs
            .split('&')
            .filter_map(|pair| {
                let mut split = pair.splitn(2, '=');
                if let (Some(key), Some(value)) = (split.next(), split.next()) {
                    Some((key.to_string(), value.to_string()))
                } else {
                    None
                }
            })
            .collect();

        RunWorkflowCommand { command, inputs: inputs_collect }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let github = GitHub::new(self.command.token.clone(), self.command.owner.clone(), self.command.repo.clone());

        let workflow = github.select_workflow().await?;

        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(&format!("Run \"{}\"({}) action in \"{}\" tree?", workflow.name, workflow.html_url, self.command.ref_name))
            .interact()?;

        if confirm {
            let url = format!("https://api.github.com/repos/{}/{}/actions/workflows/{}/dispatches", self.command.owner.clone(), self.command.repo.clone(), workflow.id);
            let _ = github.github_request(&url, "POST", Some(json!({ "ref": self.command.ref_name, "inputs": self.inputs })), None).await?;

            println!("GitHub action successfully triggered.");
            println!("Actions: https://github.com/{}/{}/actions", self.command.owner.clone(), self.command.repo.clone());
            println!("Tree: https://github.com/{}/tree/{}", self.command.repo.clone(), self.command.ref_name);

            beep(1);

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            // Get the ID of the last run.
            let runs = github.get_workflow_runs(workflow.id).await?;
            let run_id = runs.first().map(|r| r.id).ok_or("No runs found")?;

            println!("Action: https://github.com/{}/{}/actions/runs/{}", self.command.owner.clone(), self.command.repo.clone(), run_id);

            let pb = Arc::new(Mutex::new(ProgressBar::new_spinner()));
            pb.lock().await.set_style(ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner} Waiting for the workflow run to complete...").unwrap());

            let pb_clone = Arc::clone(&pb);
            tokio::spawn(async move {
                update_progress_bar(pb_clone).await;
            });
            // Wait for the workflow run to complete.
            loop {
                match github.get_workflow_run(run_id).await? {
                    Some(run) => {
                        match run.status.as_str() {
                            "completed" | "failure" => {
                                let pb = pb.lock().await;
                                pb.finish_with_message("GitHub action completed");
                                println!("");
                                println!("GitHub action completed with conclusion: {}", run.conclusion.clone().unwrap_or_else(|| "unknown".to_string()));
                                beep(3);
                                break;
                            },
                            _ => {

                            }
                        }
                    }
                    None => {

                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        } else {
            println!("{}", "Cancel".red());
        }

        Ok(())
    }
}