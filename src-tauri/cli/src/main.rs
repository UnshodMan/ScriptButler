mod commands;
mod env;
mod output;
mod resolve;
mod skill;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sbctl", bin_name = "sbctl", version, about = "ScriptButler command-line interface")]
struct Cli {
    /// Output machine-readable JSON
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Register a script in place
    Add(AddArgs),
    /// Remove a script entry (never deletes files)
    Remove(RefArgs),
    /// Copy a script into ScriptButler's managed directory
    Manage(ManageArgs),
    /// List scripts in MRU order
    List {
        /// Filter by file name or execution directory
        #[arg(long)]
        query: Option<String>,
    },
    /// Show a script entry
    Show(RefArgs),
    /// Search scripts by name, directory, and optionally content
    Search {
        keyword: String,
        /// Also scan file contents
        #[arg(long)]
        content: bool,
    },
    /// Run a script with its active or selected scheme
    Run(RunArgs),
    /// Print ScriptButler data paths
    Path,
    /// Manage the PATH environment variable
    Env {
        #[command(subcommand)]
        command: EnvCommands,
    },
    /// Print or install the ScriptButler Agent Skill
    Skill {
        /// Install into <SKILLS_ROOT>/scriptbutler/SKILL.md
        #[arg(long, value_name = "SKILLS_ROOT")]
        install: Option<PathBuf>,
    },
}

#[derive(Args)]
pub(crate) struct AddArgs {
    /// Absolute path to the script
    pub path: String,
    /// Override the display name
    #[arg(long)]
    pub name: Option<String>,
    /// Override the active scheme command template
    #[arg(long)]
    pub command: Option<String>,
    /// Name for the default run scheme
    #[arg(long = "scheme-name", default_value = "Default")]
    pub scheme_name: String,
}

#[derive(Args)]
pub(crate) struct ManageArgs {
    /// Absolute path to the script
    pub path: String,
    /// Override the display name
    #[arg(long)]
    pub name: Option<String>,
    /// Override the default scheme command template
    #[arg(long)]
    pub command: Option<String>,
    /// Execution directory for the managed copy
    #[arg(long)]
    pub workdir: Option<String>,
    /// Name for the default run scheme
    #[arg(long = "scheme-name", default_value = "Default")]
    pub scheme_name: String,
}

#[derive(Args)]
pub(crate) struct RefArgs {
    /// Script name (case-insensitive) or absolute path
    pub reference: String,
}

#[derive(Args)]
pub(crate) struct RunArgs {
    /// Script name (case-insensitive) or absolute path
    pub reference: String,
    /// Scheme id or name
    #[arg(long)]
    pub scheme: Option<String>,
}

#[derive(Subcommand)]
pub(crate) enum EnvCommands {
    /// Add the sbctl directory to PATH
    Add {
        /// Use the system PATH (Windows only, requires administrator)
        #[arg(long)]
        system: bool,
    },
    /// Remove the sbctl directory from PATH
    Remove {
        /// Use the system PATH (Windows only, requires administrator)
        #[arg(long)]
        system: bool,
    },
    /// Show whether the sbctl directory is on PATH
    Status {
        /// Check the system PATH (Windows only)
        #[arg(long)]
        system: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let json = cli.json;
    match commands::run(cli.command, json) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            output::print_error(&err, json);
            std::process::exit(err.exit_code());
        }
    }
}
