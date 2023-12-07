use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::sync::Arc;
use colored::Colorize;
use dialoguer::{Confirm, Select};
use dialoguer::theme::ColorfulTheme;
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::{Cell, format, row, Row, Table};
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;
use zip::ZipArchive;
use crate::cli::update_progress_bar;

#[derive(Deserialize, Clone)]
struct Workflow {
    id: u64,
    name: String,
    html_url: String,
}

#[derive(Deserialize, Clone)]
pub(crate) struct WorkflowRun {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) display_title: String,
    pub(crate) html_url: String,
    pub(crate) status: String,
    pub(crate) conclusion: Option<String>,
    pub(crate) head_branch: String,
    pub(crate) logs_url: String,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

pub fn unzip_and_concatenate(data_bytes: Vec<u8>) -> Result<String, Box<dyn std::error::Error>> {
    let cursor = Cursor::new(data_bytes);
    let mut archive = ZipArchive::new(cursor)?;

    let mut result = String::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_string();

        // Пропустить файлы в поддиректориях, пока не обработаем все файлы в корне
        if file_name.contains("/") {
            continue;
        }

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        result.push_str(&contents);
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_string();

        // Теперь обрабатываем только файлы в поддиректориях
        if !file_name.contains("/") {
            continue;
        }

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        result.push_str("\n");
        result.push_str("--------------\n");
        result.push_str(&file_name);
        result.push_str("\n--------------\n");
        result.push_str(&contents);
    }

    Ok(result)
}


async fn select_workflow(token: &str, owner: &str, repo: &str) -> Result<Workflow, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{}/{}/actions/workflows", owner, repo);
    let workflows_data = github_request(&url, &token, "GET", None, None).await?;
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
        .interact()?;

    Ok(workflows[selected].clone())
}

async fn select_run(token: &str, owner: &str, repo: &str, workflow_id: u64) -> Result<WorkflowRun, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{}/{}/actions/workflows/{}/runs", owner, repo, workflow_id);
    let runs_data = github_request(&url, &token, "GET", None, None).await?;
    let runs: Vec<WorkflowRun> = serde_json::from_value(runs_data["workflow_runs"].clone())?;

    let run_ids: Vec<String> = runs.iter().map(|run| {
        let id = run.id.to_string();
        let name = &run.name;
        let date = &run.created_at; // предполагая, что у вас есть поле created_at в типе WorkflowRun
        let status = match run.status.as_str() {
            "completed" => format!("{} - {} - {}", name, date, id).green().to_string(),
            "in_progress" => format!("{} - {} - {}", name, date, id).yellow().to_string(),
            "queued" => format!("{} - {} - {}", name, date, id).blue().to_string(),
            _ => format!("{} - {} - {}", name, date, id).white().to_string(),
        };
        status
    }).collect();

    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a run:")
        .items(&run_ids)
        .default(0)
        .interact()?;

    Ok(runs[selected].clone())
}

pub async fn github_request(
    url: &str,
    token: &str,
    method: &str,
    data: Option<serde_json::Value>,
    accept: Option<&str>
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let response = github_request_fn(url, token, method, data, accept).await?;
    let response_text = &response.text().await?;

    if response_text.trim().is_empty() {
        return Ok(serde_json::Value::Null);
    }

    let data: serde_json::Value = serde_json::from_str(response_text)?;
    Ok(data)
}

pub async fn github_request_bytes(
    url: &str,
    token: &str,
    method: &str,
    data: Option<serde_json::Value>,
    accept: Option<&str>
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let response = github_request_fn(url, token, method, data, accept).await?;

    let response_bytes = response.bytes().await?;

    Ok(response_bytes.to_vec())
}

pub async fn github_request_fn(
    url: &str,
    token: &str,
    method: &str,
    data: Option<serde_json::Value>,
    accept: Option<&str>
) -> Result<Response, Box<dyn std::error::Error>> {
    // let proxy = reqwest::Proxy::all("http://127.0.0.1:4034")?;
    let client = Client::builder()
        // .proxy(proxy)
        .redirect(reqwest::redirect::Policy::limited(10))
        .danger_accept_invalid_certs(true)
        .build()?;

    let accept_header = accept.unwrap_or("application/vnd.github.v3+json");

    let response = match method {
        "POST" => client.post(url)
            .header("Accept", accept_header)
            .header("Authorization", format!("token {}", token))
            .header("User-Agent", "GAR")
            .json(&data.unwrap())
            .send()
            .await?,
        _ => client.get(url)
            .header("Accept", accept_header)
            .header("Authorization", format!("token {}", token))
            .header("User-Agent", "GAR")
            .send()
            .await?,
    };

    Ok(response)
}


async fn get_workflow_runs(owner: &str, repo: &str, workflow_id: u64, token: &str) -> Result<Vec<WorkflowRun>, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{}/{}/actions/workflows/{}/runs", owner, repo, workflow_id);
    let data = github_request(&url, token, "GET", None, None).await?;
    let runs: Vec<WorkflowRun> = serde_json::from_value(data["workflow_runs"].clone())?;
    Ok(runs)
}

pub(crate) async fn get_workflow_run(owner: &str, repo: &str, run_id: u64, token: &str) -> Result<Option<WorkflowRun>, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{}/{}/actions/runs/{}", owner, repo, run_id);
    let data = github_request(&url, token, "GET", None, None).await?;
    if data.is_null() {
        Ok(None)
    } else {
        let run: WorkflowRun = serde_json::from_value(data)?;
        Ok(Some(run))
    }
}
pub(crate) async fn run_workflow(token: &str, owner: &str, repo: &str, ref_name: &str, inputs_collect: HashMap<&str, &str>) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = select_workflow(token, owner, repo).await?;

    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(&format!("Run \"{}\"({}) action in \"{}\" tree?", workflow.name, workflow.html_url, ref_name))
        .interact()?;

    if confirm {
        let url = format!("https://api.github.com/repos/{}/{}/actions/workflows/{}/dispatches", owner, repo, workflow.id);
        let _ = github_request(&url, &token, "POST", Some(json!({ "ref": ref_name, "inputs": inputs_collect })), None).await?;

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

pub(crate) async fn show_details(token: &str, owner: &str, repo: &str) -> Result<(), Box<dyn std::error::Error>> {
    let workflow = select_workflow(token, owner, repo).await?;
    let run = select_run(token, owner, repo, workflow.id).await?;

    println!("ID: {}", run.id);
    println!("Name: {}", run.name);
    println!("Display Title: {}", run.display_title);
    println!("URL: {}", run.html_url);
    println!("Status: {}", run.status);
    println!("Conclusion: {}", run.conclusion.unwrap_or_else(|| "N/A".to_string()));
    println!("Branch: {}", run.head_branch);
    println!("Created At: {}", run.created_at);
    println!("Updated At: {}", run.updated_at);

    let logs_data = github_request_bytes(run.logs_url.as_str(), &token, "GET", None, Some("application/vnd.github+json")).await?;
    let logs = unzip_and_concatenate(logs_data.clone());
    println!("Logs: \n{}", logs.unwrap());

    // https://api.github.com/repos/s00d/github-action-runner/actions/runs/7090586915/logs
    // https://api.github.com/repos/s00d/github-action-runner/actions/runs/7090586915/logs

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