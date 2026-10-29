use crate::fixer::suggest;
use crate::shell::{Shell, init_script};
use clap::{CommandFactory, Parser, Subcommand};
use std::process::ExitCode;

#[derive(Parser)]
#[command(version, about = "Fix misspelled commands", args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Cmd>,

    /// Print all suggestions instead of only the best one
    #[arg(long)]
    all: bool,

    /// The misspelled command to fix
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    words: Vec<String>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print shell integration that defines the `fuck` function
    Init {
        #[arg(value_enum)]
        shell: Shell,
    },
}

pub fn run() -> ExitCode {
    let cli = Cli::parse();
    if let Some(Cmd::Init { shell }) = cli.command {
        println!("{}", init_script(shell));
        return ExitCode::SUCCESS;
    }
    if cli.words.is_empty() {
        Cli::command().print_help().ok();
        return ExitCode::from(2);
    }

    let command = cli.words.join(" ");
    let suggestions = suggest(&command);
    if suggestions.is_empty() {
        eprintln!("fix_command: no fix found for `{command}`");
        return ExitCode::FAILURE;
    }
    if cli.all {
        suggestions.iter().for_each(|s| println!("{s}"));
    } else {
        println!("{}", suggestions[0]);
    }
    ExitCode::SUCCESS
}
