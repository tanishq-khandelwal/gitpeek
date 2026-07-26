mod app;
mod event;
mod git;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::event::{self as term_event, Event};
use std::time::Duration;

const HELP: &str = "\
git-peek - browse, preview, and pop git stashes in a terminal UI

USAGE:
    git-peek            open the stash browser in the current repo
    git-peek --help
    git-peek --version

KEYS:
    j/k, up/down     move selection
    l/right          drill into the stash's files
    h/left, Esc      back to the stash list
    Ctrl-d/Ctrl-u    scroll the diff (also PgDn/PgUp)
    Enter            pop the selected stash (asks y/n first)
    q, Ctrl-c        quit

Requires `git` on PATH. Also runnable as `git peek`.";

fn main() -> Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("--help" | "-h") => {
            println!("{HELP}");
            return Ok(());
        }
        Some("--version" | "-V") => {
            println!("git-peek {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        _ => {}
    }

    let stashes = git::list()?;
    if stashes.is_empty() {
        eprintln!("No stashes.");
        return Ok(());
    }

    let mut app = App::new(stashes);
    // ratatui::init installs a panic hook that restores the terminal first.
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();

    result?;
    if let Some(text) = app.popped {
        println!("{text}");
    }
    Ok(())
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui::render(f, app))?;
        if term_event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = term_event::read()? {
                if key.kind == term_event::KeyEventKind::Press {
                    event::handle(key, app);
                }
            }
        }
    }
    Ok(())
}
