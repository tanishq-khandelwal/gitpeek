use crate::app::{App, Confirm, Mode};
use crate::ui;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};

pub fn handle_mouse(mouse: MouseEvent, app: &mut App, area: Rect) {
    let (left, right) = ui::panes(area);
    let pos = Position::new(mouse.column, mouse.row);
    match mouse.kind {
        MouseEventKind::ScrollDown if right.contains(pos) => app.scroll_by(3),
        MouseEventKind::ScrollUp if right.contains(pos) => app.scroll_by(-3),
        MouseEventKind::ScrollDown if left.contains(pos) => app.select_stash(1),
        MouseEventKind::ScrollUp if left.contains(pos) => app.select_stash(-1),
        MouseEventKind::Down(MouseButton::Left) if app.mode == Mode::Tree && right.contains(pos) => {
            open_clicked_file(mouse, app, right);
        }
        _ => {}
    }
}

/// Row math assumes the tree isn't scrolled past the click, which holds whenever it
/// fits the pane (the common case). A taller tree can still be opened with j/k + l.
fn open_clicked_file(mouse: MouseEvent, app: &mut App, right: Rect) {
    let Some(row) = mouse.row.checked_sub(right.y + 1) else {
        return;
    };
    let line = row as usize + app.scroll as usize;
    if let Some(ui::TreeRow::File(i, _)) = ui::build_tree(&app.files, &app.file_stats).get(line) {
        app.file_i = *i;
        app.enter_file_diff();
    }
}

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

    if let Some(confirm) = app.confirm {
        match key.code {
            KeyCode::Char('y') => match confirm {
                Confirm::PopStash => app.do_pop_stash(),
                Confirm::PopFile => app.do_pop_file(),
            },
            KeyCode::Char('n') | KeyCode::Esc => app.confirm = None,
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Down | KeyCode::Char('j') => match app.mode {
            Mode::List | Mode::Tree => app.select(1),
            Mode::FileDiff => app.scroll_by(3),
        },
        KeyCode::Up | KeyCode::Char('k') => match app.mode {
            Mode::List | Mode::Tree => app.select(-1),
            Mode::FileDiff => app.scroll_by(-3),
        },
        KeyCode::PageDown => app.scroll_by(10),
        KeyCode::PageUp => app.scroll_by(-10),
        KeyCode::Enter => app.confirm = Some(Confirm::PopStash),
        KeyCode::Char('p') => match app.mode {
            Mode::Tree | Mode::FileDiff => app.confirm = Some(Confirm::PopFile),
            Mode::List => {}
        },
        KeyCode::Right | KeyCode::Char('l') => match app.mode {
            Mode::List => app.enter_tree(),
            Mode::Tree => app.enter_file_diff(),
            Mode::FileDiff => {}
        },
        KeyCode::Left | KeyCode::Char('h') => match app.mode {
            Mode::FileDiff => app.back_to_tree(),
            Mode::Tree => app.back_to_list(),
            Mode::List => {}
        },
        KeyCode::Esc => match app.mode {
            Mode::FileDiff => app.back_to_tree(),
            Mode::Tree => app.back_to_list(),
            Mode::List => app.should_quit = true,
        },
        _ => {}
    }
}
