use prettytable::{format, row, Cell, Row, Table};
use crate::github::GitHub;
use super::command::Command;

pub struct HistoryCommand {
    command: Command,
}

impl HistoryCommand {
    pub fn new(command: Command) -> Self {
        HistoryCommand { command }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let github = GitHub::new(self.command.token.clone(), self.command.owner.clone(), self.command.repo.clone());

        let workflow = github.select_workflow().await?;
        let runs = github.get_workflow_runs(workflow.id).await?;

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
}