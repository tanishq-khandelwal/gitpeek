use crate::app::{App, Mode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle(key: KeyEvent, app: &mut App) {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') => app.should_quit = true,
            KeyCode::Char('d') => app.scroll_by(10),
            KeyCode::Char('u') => app.scroll_by(-10),
            _ => {}
        }
        return;
    }

    if app.confirm {
        match key.code {
            KeyCode::Char('y') => app.do_pop(),
            KeyCode::Char('n') | KeyCode::Esc => app.confirm = false,
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Down | KeyCode::Char('j') => app.select(1),
        KeyCode::Up | KeyCode::Char('k') => app.select(-1),
        KeyCode::PageDown => app.scroll_by(10),
        KeyCode::PageUp => app.scroll_by(-10),
        KeyCode::Enter => app.confirm = true,
        KeyCode::Right | KeyCode::Char('l') if app.mode == Mode::List => app.enter_files(),
        KeyCode::Left | KeyCode::Char('h') if app.mode == Mode::Files => app.back_to_list(),
        KeyCode::Esc => match app.mode {
            Mode::Files => app.back_to_list(),
            Mode::List => app.should_quit = true,
        },
        _ => {}
    }
}
