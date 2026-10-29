# fix_command
CLI utility to fix misspelled commands (like `thefuck`).

## Usage

    cargo install --path .
    echo 'eval "$(fix_command init bash)"' >> ~/.bashrc   # or zsh / fish

    $ gti comit -m hi
    $ fuck        # runs: git commit -m hi

Or directly: `fix_command gti status` / `fix_command --all sl`.

Rules: mistyped command names (PATH lookup), git/cargo subcommands, `cd..`.
