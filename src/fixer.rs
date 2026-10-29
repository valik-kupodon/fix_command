use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;

const GIT_SUBCOMMANDS: &[&str] = &[
    "add", "bisect", "branch", "checkout", "clone", "commit", "diff", "fetch", "grep", "init",
    "log", "merge", "mv", "pull", "push", "rebase", "remote", "reset", "restore", "revert", "rm",
    "show", "stash", "status", "switch", "tag",
];

const CARGO_SUBCOMMANDS: &[&str] = &[
    "add", "bench", "build", "check", "clean", "clippy", "doc", "fmt", "init", "install", "new",
    "publish", "remove", "run", "test", "update",
];


const SHELL_BUILTINS: &[&str] = &[
    "cd", "echo", "export", "source", "alias", "exit", "pwd", "set", "unset", "type", "eval",
];


fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut prev2 = prev.clone();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                cur[j] = cur[j].min(prev2[j - 2] + 1);
            }
        }
        prev2 = prev;
        prev = cur;
    }
    prev[b.len()]
}

/// Candidates within a typo-tolerant distance, closest first.
fn closest<'a, I: IntoIterator<Item = &'a str>>(word: &str, options: I) -> Vec<String> {
    let max = if word.len() <= 3 { 1 } else { 2 };
    let mut found: Vec<(usize, String)> = options
        .into_iter()
        .filter(|o| *o != word)
        .map(|o| (levenshtein(word, o), o.to_string()))
        .filter(|(d, _)| *d <= max)
        .collect();
    found.sort();
    found.into_iter().map(|(_, o)| o).collect()
}

fn path_executables() -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    if let Some(path) = env::var_os("PATH") {
        for dir in env::split_paths(&path) {
            let Ok(entries) = fs::read_dir(dir) else { continue };
            for e in entries.flatten() {
                let is_exec = e
                    .metadata()
                    .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false);
                if let (true, Some(n)) = (is_exec, e.file_name().to_str()) {
                    set.insert(n.to_string());
                }
            }
        }
    }
    set
}

fn fix_subcommand(tool: &str, rest: &[&str]) -> Vec<String> {
    let known = match tool {
        "git" => GIT_SUBCOMMANDS,
        "cargo" => CARGO_SUBCOMMANDS,
        _ => return vec![],
    };
    let Some((sub, tail)) = rest.split_first() else { return vec![] };
    if sub.starts_with('-') {
        return vec![];
    }
    closest(sub, known.iter().copied())
        .into_iter()
        .map(|s| {
            let mut parts = vec![tool, s.as_str()];
            parts.extend_from_slice(tail);
            parts.join(" ")
        })
        .collect()
}

/// Returns suggested corrections, best first.
pub fn suggest(command: &str) -> Vec<String> {
    let words: Vec<&str> = command.split_whitespace().collect();
    let Some((first, rest)) = words.split_first() else { return vec![] };
    let mut out = Vec::new();

    // "cd.." -> "cd .."
    if let Some(dir) = first.strip_prefix("cd").filter(|d| d.starts_with("..")) {
        let mut parts = vec!["cd", dir];
        parts.extend_from_slice(rest);
        return vec![parts.join(" ")];
    }

    let execs = path_executables();
    if execs.contains(*first) || SHELL_BUILTINS.contains(first) {
        out.extend(fix_subcommand(first, rest));
    } else {
        for c in closest(first, execs.iter().map(String::as_str)) {
            // A corrected tool with a mistyped subcommand: fix both at once.
            out.extend(fix_subcommand(&c, rest));
            let mut parts = vec![c.as_str()];
            parts.extend_from_slice(rest);
            out.push(parts.join(" "));
        }
    }

    let mut seen = BTreeSet::new();
    out.retain(|s| seen.insert(s.clone()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("ls", "sl"), 1);
    }

    #[test]
    fn cd_dots() {
        assert_eq!(suggest("cd.."), vec!["cd .."]);
    }

    #[test]
    fn git_subcommand() {
        assert_eq!(suggest("git comit -m hi")[0], "git commit -m hi");
    }

    #[test]
    fn tool_typo() {
        assert_eq!(suggest("gti status")[0], "git status");
    }

    #[test]
    fn tool_and_subcommand_typo() {
        assert_eq!(suggest("gti comit")[0], "git commit");
    }
}
