# lazystash

[![ci](https://github.com/tanishq-khandelwal/lazystash/actions/workflows/ci.yml/badge.svg)](https://github.com/tanishq-khandelwal/lazystash/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/lazystash.svg)](https://crates.io/crates/lazystash)
[![npm](https://img.shields.io/npm/v/lazystash.svg)](https://www.npmjs.com/package/lazystash)
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
cargo install lazystash     # Rust toolchain
npm install -g lazystash    # downloads the prebuilt binary for your platform
```

Or download a prebuilt archive from [Releases](https://github.com/tanishq-khandelwal/lazystash/releases) and put `lazystash` on your PATH:

```sh
tar xzf lazystash-aarch64-apple-darwin.tar.gz
mv lazystash /usr/local/bin/
```

Prebuilt targets: macOS (Intel + Apple Silicon), Linux (x86_64 + arm64, static musl), Windows (x86_64). Each archive ships a `.sha256` alongside it.

The npm package contains no binary — it's a ~2kB wrapper whose postinstall downloads the release archive for your platform and verifies it against the published checksum before unpacking. So it needs network access to GitHub at install time, and `--ignore-scripts` will skip it (run `node node_modules/lazystash/install.js` yourself in that case).

## Usage

```sh
lazystash
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

Requires `git` on your PATH; it is the only runtime dependency.

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

1. Bump `version` in **both** `Cargo.toml` and `npm/package.json` (CI fails the release if they and the tag disagree — `install.js` builds its download URL from its own version, so a stale one 404s for every installer).
2. Commit, then tag and push:
   ```sh
   git tag v0.1.0 && git push --tags
   ```

The release workflow then verifies the tag matches both manifests, cross-builds all five targets, uploads the archives + checksums to a GitHub Release, and only then publishes to crates.io and npm. Publishing is last on purpose: both registries let you unpublish or yank but never replace a version, so nothing ships unless every target built. The npm job additionally runs its own `install.js` against the fresh release and executes the binary before publishing.

One-time setup — add both as repository secrets (Settings → Secrets and variables → Actions):

| Secret | Where to get it |
| --- | --- |
| `CARGO_REGISTRY_TOKEN` | crates.io → Account Settings → API Tokens |
| `NPM_TOKEN` | npmjs.com → Access Tokens → Classic → **Automation** |

`NPM_TOKEN` must be an *Automation* token. A classic *Publish* token, or a granular token without 2FA bypass, returns `403 ... two-factor authentication ... is required` in CI, because there is no way to answer an OTP prompt there.

The repository must be **public** for the npm channel to work: `install.js` downloads release assets over plain HTTPS, and a private repo's assets require authentication, so every `npm install -g lazystash` would fail.

To publish npm by hand instead: `cd npm && npm version <ver> && npm publish` (after `npm login`).

## Not included (deliberately)

Syntax highlighting inside diffs, `apply`/`drop`/`branch` actions, creating stashes, fuzzy filtering, mouse support. `pop` covers the actual use case; the rest is weight.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
