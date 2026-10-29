mod cli;
mod fixer;
mod shell;

fn main() -> std::process::ExitCode {
    cli::run()
}
