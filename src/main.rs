mod app;
mod event;
mod git;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::event::{self as term_event, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::execute;
use ratatui::layout::Rect;
use std::io::stdout;
use std::time::Duration;

const HELP: &str = "\
lazystash - browse, preview, and pop git stashes in a terminal UI

USAGE:
    lazystash            open the stash browser in the current repo
    lazystash --help
    lazystash --version

KEYS:
    j/k, up/down     move selection (scrolls instead, while viewing a file's diff)
    l/right          drill in: stash -> file tree -> file diff
    h/left, Esc      go back a level
    Ctrl-d/Ctrl-u    scroll (also PgDn/PgUp)
    Enter            pop the selected stash (asks y/n first)
    p                pop just the selected file, leaving the rest stashed (in file tree/diff)
    q, Ctrl-c        quit

Requires `git` on PATH.";

fn main() -> Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("--help" | "-h") => {
            println!("{HELP}");
            return Ok(());
        }
        Some("--version" | "-V") => {
            println!("lazystash {}", env!("CARGO_PKG_VERSION"));
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
    execute!(stdout(), EnableMouseCapture)?;
    let result = run(&mut terminal, &mut app);
    execute!(stdout(), DisableMouseCapture).ok();
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
            match term_event::read()? {
                Event::Key(key) if key.kind == term_event::KeyEventKind::Press => {
                    event::handle(key, app);
                }
                Event::Mouse(mouse) => {
                    let size = terminal.size()?;
                    let area = Rect::new(0, 0, size.width, size.height);
                    event::handle_mouse(mouse, app, area);
                }
                _ => {}
            }
        }
    }
    Ok(())
}
