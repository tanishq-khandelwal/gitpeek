# gitpeek

[![ci](https://github.com/tanishq-khandelwal/gitpeek/actions/workflows/ci.yml/badge.svg)](https://github.com/tanishq-khandelwal/gitpeek/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/gitpeek.svg)](https://crates.io/crates/gitpeek)
[![license](https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg)](LICENSE)

Browse, preview, and pop `git stash` entries in a terminal UI. One static binary, no fzf, no libgit2 — just `git` on your PATH.

Left pane: your stashes. Right pane: the live, colorized diff of whatever's highlighted. Drill into a single stash's files, then pop it once you've found the one you wanted.

```
┌git stashes───────────────────────────┐┌diff─────────────────────────────────────────┐
│stash@{0}  [main] second wip          ││diff --git a/a.txt b/a.txt                   │
│stash@{1}  [main] first wip           ││index 5626abf..f719efd 100644                │
│                                      ││--- a/a.txt                                  │
│                                      ││+++ b/a.txt                                  │
│                                      ││@@ -1 +1 @@                                  │
│                                      ││-one                                         │
│                                      ││+two                                         │
└──────────────────────────────────────┘└─────────────────────────────────────────────┘
j/k move  l files  Enter pop  Ctrl-u/d scroll  q quit
```

## Install

```sh
cargo install gitpeek
```

Or download a prebuilt archive from [Releases](https://github.com/tanishq-khandelwal/gitpeek/releases) and put `git-peek` on your PATH:

```sh
tar xzf gitpeek-aarch64-apple-darwin.tar.gz
mv git-peek /usr/local/bin/
```

Prebuilt targets: macOS (Intel + Apple Silicon), Linux (x86_64 + arm64, static musl), Windows (x86_64). Each archive ships a `.sha256` alongside it.

## Usage

```sh
git-peek        # or: git peek
```

| Key | Action |
| --- | --- |
| `j` / `k`, `↑` / `↓` | move selection |
| `l` / `→` | drill into the selected stash's files |
| `h` / `←`, `Esc` | back to the stash list |
| `Ctrl-d` / `Ctrl-u`, `PgDn` / `PgUp` | scroll the diff |
| `Enter` | pop the selected stash (asks `y`/`n` first) |
| `q`, `Ctrl-c` | quit |

`--help` and `--version` print and exit without starting the UI. With no stashes it prints `No stashes.` and exits 0; outside a git repo it prints git's error and exits 1.

Pop is the only action that mutates anything, and it always confirms first. Conflict output from `git stash pop` is printed after the UI closes, so you can read it.

Requires `git`. Because the binary is named `git-peek`, git picks it up as the `git peek` subcommand automatically — no alias or config needed.

## Development

```sh
cargo test                                # unit + repo-backed integration tests
cargo clippy --all-targets -- -D warnings
cargo fmt --all
cargo run                                 # run against the current repo's stashes
```

Layout: `git.rs` (all `git` shell-outs + parsing), `app.rs` (state + transitions), `ui.rs` (rendering), `event.rs` (keys → state), `main.rs` (args, terminal setup/teardown).

The test suite creates a real scratch repo in your temp dir, makes two stashes, and exercises the git layer and the state machine against it — including an actual `pop`. Repo-backed tests all live in a single `#[test]` because they `set_current_dir`, which is process-global.

CI runs fmt, clippy, tests, and a release build on Linux, macOS, and Windows, plus `cargo publish --dry-run`.

## Releasing

1. Bump `version` in `Cargo.toml`.
2. Commit, then tag and push:
   ```sh
   git tag v0.1.0 && git push --tags
   ```

The release workflow then verifies the tag matches `Cargo.toml`, cross-builds all five targets, uploads the archives + checksums to a GitHub Release, and finally runs `cargo publish`. Publishing is last on purpose: a crates.io version can be yanked but never replaced, so nothing is published unless every target built.

One-time setup: add a crates.io API token as the `CARGO_REGISTRY_TOKEN` secret under a `crates-io` GitHub environment.

## Not included (deliberately)

Syntax highlighting inside diffs, `apply`/`drop`/`branch` actions, creating stashes, fuzzy filtering, mouse support. `pop` covers the actual use case; the rest is weight.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
