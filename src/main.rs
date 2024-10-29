mod commands;
mod github;
mod git;
mod helpers;

use crate::commands::{command::Command as BaseCommand, history::HistoryCommand, details::DetailsCommand, autocomplete::AutocompleteCommand, run_workflow::RunWorkflowCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gar_command = BaseCommand::create_command();
    let matches = gar_command.get_matches();

    let base_command = BaseCommand::new(&matches);

    match matches.subcommand() {
        Some(("history", _)) => {
            let history_command = HistoryCommand::new(base_command);
            history_command.run().await?;
            return Ok(());
        }
        Some(("details", _)) => {
            let details_command = DetailsCommand::new(base_command);
            details_command.run().await?;
            return Ok(());
        }
        Some(("autocomplete", _)) => {
            let autocomplete_command = AutocompleteCommand::new(base_command);
            autocomplete_command.run()?;
            return Ok(());
        }
        _ => {
            let inputs = matches.get_one::<String>("inputs").map(|s| s.to_owned()).unwrap_or_else(String::new);
            let run_workflow_command = RunWorkflowCommand::new(base_command, inputs);
            run_workflow_command.run().await?;
        },
    };

    Ok(())
}