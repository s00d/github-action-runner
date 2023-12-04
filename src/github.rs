use std::sync::Arc;
use colored::Colorize;
use dialoguer::{Confirm, Select};
use dialoguer::theme::ColorfulTheme;
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::{Cell, format, row, Row, Table};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;
use crate::cli::update_progress_bar;

#[derive(Deserialize, Clone)]
struct Workflow {
    id: u64,
    name: String,
    html_url: String,
}

#[derive(Deserialize)]
struct WorkflowRun {
    id: u64,
    html_url: String,
    status: String,
    conclusion: Option<String>,
    head_branch: String,
    created_at: String,
    updated_at: String,
}


async fn select_workflow(token: &str, owner: &str, repo: &str) -> Result<Workflow, Box<dyn std::error::Error>> {
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

    Ok(workflows[selected].clone())
}

async fn github_request(url: &str, token: &str, method: &str, data: Option<serde_json::Value>) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
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

    if response_text.trim().is_empty() {
        return Ok(serde_json::Value::Null);
    }

    let data: serde_json::Value = serde_json::from_str(&response_text)?;
    Ok(data)
}


async fn get_workflow_runs(owner: &str, repo: &str, workflow_id: u64, token: &str) -> Result<Vec<WorkflowRun>, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{}/{}/actions/workflows/{}/runs", owner, repo, workflow_id);
    let data = github_request(&url, token, "GET", None).await?;
    let runs: Vec<WorkflowRun> = serde_json::from_value(data["workflow_runs"].clone())?;
    Ok(runs)
}

async fn get_workflow_run(owner: &str, repo: &str, run_id: u64, token: &str) -> Result<Option<WorkflowRun>, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{}/{}/actions/runs/{}", owner, repo, run_id);
    let data = github_request(&url, token, "GET", None).await?;
    if data.is_null() {
        Ok(None)
    } else {
        let run: WorkflowRun = serde_json::from_value(data)?;
        Ok(Some(run))
    }
}
pub(crate) async fn run_workflow(token: &str, owner: &str, repo: &str, ref_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = select_workflow(token, owner, repo).await?;

    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(&format!("Run \"{}\"({}) action in \"{}\" tree?", workflow.name, workflow.html_url, ref_name))
        .interact()
        .unwrap();

    if confirm {
        let url = format!("https://api.github.com/repos/{}/{}/actions/workflows/{}/dispatches", owner, repo, workflow.id);
        let _ = github_request(&url, &token, "POST", Some(json!({ "ref": ref_name }))).await?;

        println!("GitHub action successfully triggered.");
        println!("Actions: https://github.com/{}/{}/actions", owner, repo);
        println!("Tree: https://github.com/{}/tree/{}", repo, ref_name);

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        // Get the ID of the last run.
        let runs = get_workflow_runs(owner, repo, workflow.id, token).await?;
        let run_id = runs.first().map(|r| r.id).ok_or("No runs found")?;

        println!("Action: https://github.com/{}/{}/actions/runs/{}", owner, repo, run_id);

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
            match get_workflow_run(owner, repo, run_id, token).await? {
                Some(run) => {
                    match run.status.as_str() {
                        "completed" | "failure" => {
                            let pb = pb.lock().await;
                            pb.finish_with_message("GitHub action completed");
                            println!("");
                            println!("GitHub action completed with conclusion: {}", run.conclusion.clone().unwrap_or_else(|| "unknown".to_string()));
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


pub(crate) async fn show_history(token: &str, owner: &str, repo: &str) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = select_workflow(token, owner, repo).await?;
    let runs = get_workflow_runs(owner, repo, workflow.id, token).await?;

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.add_row(row!["ID", "Branch", "Status", "Conclusion", "Created At", "Updated At", "Url"]);

    for run in runs.iter().take(10) {
        let conclusion = match &run.conclusion {
            Some(value) => value.as_str(),
            None => "N/A",
        };

        table.add_row(Row::new(vec![
            Cell::new(&run.id.to_string()),
            Cell::new(&run.head_branch),
            Cell::new(&run.status),
            Cell::new(conclusion),
            Cell::new(&run.created_at),
            Cell::new(&run.updated_at),
            Cell::new(&run.html_url),
        ]));
    }

    table.printstd();

    Ok(())
}