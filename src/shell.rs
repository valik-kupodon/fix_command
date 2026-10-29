use clap::ValueEnum;
use std::env;

#[derive(Clone, Copy, ValueEnum)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

// The function body runs inside the user's shell, so `fc`/`history` see the real last command.
// Inside the function the current `fuck` line is already in bash/zsh history, hence `-2`.
pub fn init_script(shell: Shell) -> String {
    let bin = env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| "fix_command".into());
    match shell {
        Shell::Bash | Shell::Zsh => format!(
            "fuck() {{\n  local cmd fixed\n  cmd=$(fc -ln -2 | head -n 1 | sed 's/^[[:space:]]*//')\n  \
             fixed=$('{bin}' \"$cmd\") || return\n  \
             echo \"$fixed\" >&2\n  eval \"$fixed\"\n}}"
        ),
        Shell::Fish => format!(
            "function fuck\n  set -l cmd $history[1]\n  \
             set -l fixed ('{bin}' \"$cmd\")\n  or return\n  \
             echo $fixed >&2\n  eval $fixed\nend"
        ),
    }
}

