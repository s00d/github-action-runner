use colored::Colorize;
use dialoguer::{Select};
use dialoguer::theme::ColorfulTheme;
use reqwest::{Client, Response};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Workflow {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) html_url: String,
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

pub struct GitHub {
    token: String,
    owner: String,
    repo: String,
}

impl GitHub {
    pub fn new(token: String, owner: String, repo: String) -> GitHub {
        GitHub { token, owner, repo }
    }

    pub(crate) async fn select_workflow(&self) -> Result<Workflow, Box<dyn std::error::Error>> {
        let url = format!("https://api.github.com/repos/{}/{}/actions/workflows", self.owner, self.repo);
        let workflows_data = self.github_request(&url, "GET", None, None)
            .await?;
        let workflows: Vec<Workflow> = serde_json::from_value(workflows_data["workflows"].clone())
            .map_err(|e| format!("Bad request, check token or permissions. Original error: {}", e))?;

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

    pub(crate) async fn select_run(&self, workflow_id: u64) -> Result<WorkflowRun, Box<dyn std::error::Error>> {
        let url = format!("https://api.github.com/repos/{}/{}/actions/workflows/{}/runs", self.owner, self.repo, workflow_id);
        let runs_data = self.github_request(&url, "GET", None, None).await?;
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
        &self,
        url: &str,
        method: &str,
        data: Option<serde_json::Value>,
        accept: Option<&str>
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let response = self.github_request_fn(url, method, data, accept).await?;
        let response_text = &response.text().await?;

        if response_text.trim().is_empty() {
            return Ok(serde_json::Value::Null);
        }

        let data: serde_json::Value = serde_json::from_str(response_text)?;
        Ok(data)
    }

    pub async fn github_request_bytes(
        &self,
        url: &str,
        method: &str,
        data: Option<serde_json::Value>,
        accept: Option<&str>
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let response = self.github_request_fn(url, method, data, accept).await?;

        let response_bytes = response.bytes().await?;

        Ok(response_bytes.to_vec())
    }

    pub async fn github_request_fn(
        &self,
        url: &str,
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
                .header("Authorization", format!("token {}", self.token))
                .header("User-Agent", "GAR")
                .json(&data.unwrap())
                .send()
                .await?,
            _ => client.get(url)
                .header("Accept", accept_header)
                .header("Authorization", format!("token {}", self.token))
                .header("User-Agent", "GAR")
                .send()
                .await?,
        };

        Ok(response)
    }


    pub(crate) async fn get_workflow_runs(&self, workflow_id: u64) -> Result<Vec<WorkflowRun>, Box<dyn std::error::Error>> {
        let url = format!("https://api.github.com/repos/{}/{}/actions/workflows/{}/runs", self.owner, self.repo, workflow_id);
        let data = self.github_request(&url, "GET", None, None).await?;
        let runs: Vec<WorkflowRun> = serde_json::from_value(data["workflow_runs"].clone())?;
        Ok(runs)
    }

    pub(crate) async fn get_workflow_run(&self, run_id: u64) -> Result<Option<WorkflowRun>, Box<dyn std::error::Error>> {
        let url = format!("https://api.github.com/repos/{}/{}/actions/runs/{}", self.owner, self.repo, run_id);
        let data = self.github_request(&url, "GET", None, None).await?;
        if data.is_null() {
            Ok(None)
        } else {
            let run: WorkflowRun = serde_json::from_value(data)?;
            Ok(Some(run))
        }
    }
}
