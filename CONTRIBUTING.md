# Contributing

Thanks for taking a look. Bug reports and small, focused pull requests are both welcome.

## Branches

`develop` is the default branch and where all work lands. `main` only ever receives release merges, so **open your pull request against `develop`**.

Both branches are protected: changes arrive by pull request, and releases are cut by the maintainer.

## Getting set up

You need a Rust toolchain ([rustup](https://rustup.rs)) and `git` on your PATH. Nothing else.

```sh
git clone https://github.com/tanishq-khandelwal/lazystash.git
cd lazystash
cargo run          # runs against the current repo's stashes
```

## Before opening a pull request

Run what CI runs, so you find problems before it does:

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

All three must be clean. CI repeats them on Linux, macOS, and Windows.

## Tests

The suite shells out to real `git`: it creates a scratch repo in your temp dir, makes two stashes, and exercises the git layer and the state machine against it — including an actual `pop`.

Two things to know if you add tests there:

- Repo-backed tests live in a **single** `#[test]` because they call `set_current_dir`, which is process-global and would race across parallel test threads.
- The fixture sets `core.autocrlf=false`. Without it, Windows runners rewrite the LF fixtures as CRLF and byte-exact assertions fail.

## Code layout

| File | Responsibility |
| --- | --- |
| `src/git.rs` | every `git` shell-out, plus stash-line parsing |
| `src/app.rs` | application state and transitions |
| `src/ui.rs` | rendering both panes, footer, and the confirm modal |
| `src/event.rs` | key events → state changes |
| `src/main.rs` | args, terminal setup/teardown, run loop |
| `npm/` | the npm wrapper that downloads a prebuilt binary |

## Regenerating the demo

The README GIF is scripted, not hand-recorded, so it can be rebuilt whenever the UI changes. It needs [`vhs`](https://github.com/charmbracelet/vhs) (`brew install vhs`):

```sh
cargo build --release
./demo/seed.sh                      # synthetic repo with five fake stashes in /tmp/lazystash-demo
PATH="$PWD/target/release:$PATH" vhs demo/demo.tape
```

That writes `assets/demo.gif`. The seed repo is entirely synthetic on purpose — never record a demo against a real work repository, since stash messages and diffs end up published in the README.

## Scope

The tool deliberately stays small: browse stashes, read their diffs, pop one. Syntax highlighting, `apply`/`drop`/`branch` actions, stash creation, fuzzy filtering, and mouse support are all out of scope — please open an issue to discuss before building any of them.

There is one hard rule: `pop` is the only operation that may mutate a repository, and it must always confirm first.
