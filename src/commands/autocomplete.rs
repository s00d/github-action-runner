use std::{env, fs};
use super::command::Command;

pub struct AutocompleteCommand {
    _command: Command,
}

impl AutocompleteCommand {
    pub fn new(_command: Command) -> Self {
        AutocompleteCommand { _command }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {


        let current_dir = env::current_dir()?;
        let source_path = current_dir.join("completions/zsh");
        let dest_dir = dirs::home_dir().ok_or("Could not get home directory")?.join(".oh-my-zsh/plugins/gar/");

        // Ensure the destination directory exists
        fs::create_dir_all(&dest_dir)?;

        let dest_path = dest_dir.join("_gar");

        // Copy the file
        fs::copy(source_path, &dest_path)?;

        println!("Zsh autocompletion installed at {:?}", dest_path);

        let zsh_config = dirs::home_dir().ok_or("Could not get home directory")?.join(".zsh");

        println!("you need add plugin `gar` to your zsh config {:?}", zsh_config);

        Ok(())
    }
}