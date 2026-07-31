use anyhow::{bail, Context, Result};
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub struct Stash {
    pub reference: String,
    pub branch: String,
    pub message: String,
}

fn run(args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .context("failed to run `git` - is it installed and on PATH?")?;
    if !out.status.success() {
        bail!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Parse one `git stash list` line. Tolerant: unrecognised tails land in `message`.
fn parse_line(line: &str) -> Option<Stash> {
    let (reference, tail) = line.split_once(": ")?;
    let (branch, message) = tail
        .strip_prefix("WIP on ")
        .or_else(|| tail.strip_prefix("On "))
        .and_then(|rest| rest.split_once(": "))
        .map(|(b, m)| (b.to_string(), m.to_string()))
        .unwrap_or_else(|| (String::new(), tail.to_string()));
    Some(Stash {
        reference: reference.to_string(),
        branch,
        message,
    })
}

pub fn list() -> Result<Vec<Stash>> {
    Ok(run(&["stash", "list"])?
        .lines()
        .filter_map(parse_line)
        .collect())
}

pub fn diff(reference: &str) -> Result<String> {
    run(&["stash", "show", "-p", reference])
}

/// `git stash show --numstat` lines look like "<added>\t<deleted>\t<path>"; binary
/// files show "-\t-\t<path>", which collapses to 0/0 here.
pub fn file_stats(reference: &str) -> Result<Vec<(String, u32, u32)>> {
    Ok(run(&["stash", "show", "--numstat", reference])?
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|l| {
            let mut parts = l.splitn(3, '\t');
            let added = parts.next()?.parse().unwrap_or(0);
            let deleted = parts.next()?.parse().unwrap_or(0);
            let path = parts.next()?.to_string();
            Some((path, added, deleted))
        })
        .collect())
}

/// `git stash show` rejects a pathspec ("Too many revisions specified"), so diff the
/// stash commit against its first parent instead - same output, one file.
pub fn file_diff(reference: &str, path: &str) -> Result<String> {
    run(&["diff", &format!("{reference}^1"), reference, "--", path])
}

/// `git stash pop`. Returns stdout+stderr so the caller can surface conflicts.
pub fn pop(reference: &str) -> Result<String> {
    let out = Command::new("git")
        .args(["stash", "pop", reference])
        .output()
        .context("failed to run `git`")?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if !out.status.success() {
        bail!("{}", text.trim());
    }
    Ok(text)
}

/// Git has no native partial-pop, so this pops the whole stash, then re-stashes every
/// other file from it under a new entry. If the re-stash fails, the pop already
/// succeeded - the error says the rest is sitting unstashed rather than losing it.
pub fn pop_file(reference: &str, path: &str, all_files: &[String]) -> Result<String> {
    let others: Vec<&str> = all_files
        .iter()
        .map(String::as_str)
        .filter(|f| *f != path)
        .collect();

    let pop_out = pop(reference)?;
    if others.is_empty() {
        return Ok(pop_out);
    }

    let msg = format!("rest of {reference}, after popping {path}");
    let mut args = vec!["stash", "push", "-u", "-m", msg.as_str(), "--"];
    args.extend(others);

    let out = Command::new("git")
        .args(&args)
        .output()
        .context("failed to run `git`")?;
    let restash_text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if !out.status.success() {
        bail!(
            "popped {path}, but the rest of the stash is now unstashed in your working tree - re-stash it yourself: {}",
            restash_text.trim()
        );
    }
    Ok(format!(
        "{}\nre-stashed the rest: {}",
        pop_out.trim_end(),
        restash_text.trim()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stash_list_forms() {
        let wip = parse_line("stash@{0}: WIP on main: 1a2b3c4 tidy up").unwrap();
        assert_eq!(wip.reference, "stash@{0}");
        assert_eq!(wip.branch, "main");
        assert_eq!(wip.message, "1a2b3c4 tidy up");

        let on = parse_line("stash@{1}: On feat/x: my message").unwrap();
        assert_eq!(on.reference, "stash@{1}");
        assert_eq!(on.branch, "feat/x");
        assert_eq!(on.message, "my message");

        // branch names may contain ": " - first split wins, remainder is the message
        let odd = parse_line("stash@{2}: something unexpected").unwrap();
        assert_eq!(odd.reference, "stash@{2}");
        assert_eq!(odd.branch, "");
        assert_eq!(odd.message, "something unexpected");

        // message may itself contain ": " - only the first split separates the reference
        let colons = parse_line("stash@{3}: On main: fix: the thing").unwrap();
        assert_eq!(colons.branch, "main");
        assert_eq!(colons.message, "fix: the thing");

        assert_eq!(parse_line("garbage"), None);
        assert_eq!(parse_line(""), None);
    }
}

/// Repo-backed tests. All of them live in ONE `#[test]` because they `set_current_dir`,
/// which is process-global and would race against parallel test threads.
#[cfg(test)]
mod repo_tests {
    use super::*;
    use crate::app::{App, Mode};
    use std::fs;
    use std::path::PathBuf;

    fn git(dir: &PathBuf, args: &[&str]) {
        let out = Command::new("git")
            .current_dir(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// Two stashes: stash@{0} touches one file, stash@{1} touches two.
    fn scratch_repo() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lazystash-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q", "--initial-branch=main", "."]);
        git(&dir, &["config", "user.email", "test@example.com"]);
        git(&dir, &["config", "user.name", "test"]);
        // Windows runners default to autocrlf=true, which would rewrite the LF
        // fixtures as CRLF and break the byte-exact assertions below.
        git(&dir, &["config", "core.autocrlf", "false"]);
        fs::write(dir.join("a.txt"), "one\n").unwrap();
        fs::write(dir.join("b.txt"), "x\n").unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-qm", "init"]);

        fs::write(dir.join("a.txt"), "one changed\n").unwrap();
        fs::write(dir.join("b.txt"), "y\n").unwrap();
        git(&dir, &["stash", "-q", "-m", "first wip"]);
        fs::write(dir.join("a.txt"), "two\n").unwrap();
        git(&dir, &["stash", "-q", "-m", "second wip"]);
        dir
    }

    #[test]
    fn end_to_end_against_a_real_repo() {
        let dir = scratch_repo();
        std::env::set_current_dir(&dir).unwrap();

        // --- git.rs ---
        let stashes = list().unwrap();
        assert_eq!(stashes.len(), 2, "newest first");
        assert_eq!(stashes[0].reference, "stash@{0}");
        assert_eq!(stashes[0].branch, "main");
        assert_eq!(stashes[0].message, "second wip");

        let d = diff("stash@{0}").unwrap();
        assert!(d.contains("a.txt") && d.contains("+two"), "{d}");

        let stats = file_stats("stash@{1}").unwrap();
        assert_eq!(
            stats.iter().map(|(p, ..)| p.clone()).collect::<Vec<_>>(),
            vec!["a.txt", "b.txt"]
        );

        // Regression: `git stash show -p <ref> -- <path>` is rejected by git, so this
        // must diff against the stash's first parent instead.
        let only_b = file_diff("stash@{1}", "b.txt").unwrap();
        assert!(only_b.contains("+y"), "{only_b}");
        assert!(
            !only_b.contains("a.txt"),
            "should be scoped to b.txt: {only_b}"
        );

        assert!(
            diff("stash@{99}").is_err(),
            "bad ref must error, not return empty"
        );

        // --- app.rs state machine ---
        let mut app = App::new(list().unwrap());
        assert!(!app.diff.is_empty());
        assert_eq!(app.list_i, 0);

        app.select(-1);
        assert_eq!(app.list_i, 0, "clamps at the top");
        app.select(1);
        assert_eq!(app.list_i, 1);
        assert!(
            app.diff.iter().any(|l| l.contains("+one changed")),
            "diff followed selection"
        );
        app.select(5);
        assert_eq!(app.list_i, 1, "clamps at the bottom");

        app.scroll_by(-1);
        assert_eq!(app.scroll, 0, "clamps at the top");
        app.scroll_by(10_000);
        assert_eq!(
            app.scroll as usize,
            app.diff.len() - 1,
            "clamps at the last line"
        );

        app.enter_tree();
        assert_eq!(app.mode, Mode::Tree);
        assert_eq!(app.files, vec!["a.txt", "b.txt"]);
        assert_eq!(app.file_stats.len(), 2);
        assert!(
            app.diff.is_empty(),
            "tree view has no diff until a file is opened"
        );

        app.select(1);
        assert_eq!(app.file_i, 1);

        app.enter_file_diff();
        assert_eq!(app.mode, Mode::FileDiff);
        assert_eq!(app.scroll, 0, "scroll resets on refresh");
        assert!(app.diff.iter().any(|l| l.contains("+y")), "per-file diff");

        app.back_to_tree();
        assert_eq!(app.mode, Mode::Tree);
        assert!(app.diff.is_empty());

        app.back_to_list();
        assert_eq!(app.mode, Mode::List);
        assert!(app.files.is_empty());

        // --- pop mutates the stack ---
        app.list_i = 0;
        app.refresh_diff();
        app.do_pop_stash();
        assert!(app.should_quit);
        assert!(app.popped.unwrap().contains("Popped stash@{0}"));
        assert_eq!(list().unwrap().len(), 1, "one stash consumed");
        assert_eq!(
            fs::read_to_string(dir.join("a.txt")).unwrap(),
            "two\n",
            "changes restored"
        );

        let _ = fs::remove_dir_all(&dir);

        // --- pop_file: pull one file out of a multi-file stash, leave the rest stashed ---
        let dir2 = two_file_stash_repo();
        std::env::set_current_dir(&dir2).unwrap();

        let files = file_stats("stash@{0}").unwrap();
        let paths: Vec<String> = files.iter().map(|(p, ..)| p.clone()).collect();
        assert_eq!(paths, vec!["a.txt", "b.txt"]);

        pop_file("stash@{0}", "b.txt", &paths).unwrap();

        assert_eq!(
            fs::read_to_string(dir2.join("b.txt")).unwrap(),
            "b2\n",
            "popped file's change lands in the working tree"
        );
        assert_eq!(
            fs::read_to_string(dir2.join("a.txt")).unwrap(),
            "a1\n",
            "other file stays untouched - its change is still stashed, not applied"
        );

        let remaining = list().unwrap();
        assert_eq!(
            remaining.len(),
            1,
            "original stash replaced by a new one holding the rest"
        );
        let rest_diff = diff(&remaining[0].reference).unwrap();
        assert!(
            rest_diff.contains("a.txt") && rest_diff.contains("+a2"),
            "a.txt's change preserved in the new stash: {rest_diff}"
        );
        assert!(
            !rest_diff.contains("b.txt"),
            "b.txt must not reappear in the re-stashed remainder: {rest_diff}"
        );

        let _ = fs::remove_dir_all(&dir2);
    }

    /// One stash touching two files, for testing `pop_file`'s partial-pop behavior.
    fn two_file_stash_repo() -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("lazystash-popfile-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q", "--initial-branch=main", "."]);
        git(&dir, &["config", "user.email", "test@example.com"]);
        git(&dir, &["config", "user.name", "test"]);
        git(&dir, &["config", "core.autocrlf", "false"]);
        fs::write(dir.join("a.txt"), "a1\n").unwrap();
        fs::write(dir.join("b.txt"), "b1\n").unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-qm", "init"]);
        fs::write(dir.join("a.txt"), "a2\n").unwrap();
        fs::write(dir.join("b.txt"), "b2\n").unwrap();
        git(&dir, &["stash", "-q", "-m", "two files"]);
        dir
    }
}
