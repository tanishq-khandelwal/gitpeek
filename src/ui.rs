use crate::app::{App, Confirm, Mode};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, List, ListState, Paragraph};
use ratatui::Frame;

/// A row in the file tree: a non-selectable directory header, or a file
/// (carrying its index into `App::files`/`App::file_stats`).
pub enum TreeRow {
    Dir(String),
    File(usize, String),
}

/// Groups files by shared directory prefixes, printing each directory header only
/// once - just enough structure to read as a tree. No expand/collapse, since stash
/// file lists are small enough that a flat grouped view already fits.
pub fn build_tree(files: &[String], stats: &[(u32, u32)]) -> Vec<TreeRow> {
    let mut rows = Vec::new();
    let mut prev_dirs: Vec<&str> = Vec::new();
    for (i, path) in files.iter().enumerate() {
        let parts: Vec<&str> = path.split('/').collect();
        let dirs = &parts[..parts.len() - 1];
        let name = parts[parts.len() - 1];
        let common = dirs
            .iter()
            .zip(prev_dirs.iter())
            .take_while(|(a, b)| a == b)
            .count();
        for (depth, d) in dirs.iter().enumerate().skip(common) {
            rows.push(TreeRow::Dir(format!("{}{d}/", "  ".repeat(depth))));
        }
        let (added, deleted) = stats.get(i).copied().unwrap_or((0, 0));
        rows.push(TreeRow::File(
            i,
            format!("{}{name}  +{added} -{deleted}", "  ".repeat(dirs.len())),
        ));
        prev_dirs = dirs.to_vec();
    }
    rows
}

/// Left/right pane rects, shared with mouse hit-testing so scroll/click events
/// land on whichever pane the cursor is actually over.
pub fn panes(area: Rect) -> (Rect, Rect) {
    let [main, _footer] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
    Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
        .areas(main)
        .into()
}

pub fn render(f: &mut Frame, app: &App) {
    let [main, footer] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(f.area());
    let (left, right) = panes(f.area());

    let reference = app.current().map(|s| s.reference.as_str()).unwrap_or("-");

    let stash_items: Vec<String> = app
        .stashes
        .iter()
        .map(|s| {
            if s.branch.is_empty() {
                format!("{}  {}", s.reference, s.message)
            } else {
                format!("{}  [{}] {}", s.reference, s.branch, s.message)
            }
        })
        .collect();
    let stash_list = List::new(stash_items)
        .block(Block::bordered().title("git stashes"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_stateful_widget(
        stash_list,
        left,
        &mut ListState::default().with_selected(Some(app.list_i)),
    );

    let title;
    let lines: Vec<Line> = match app.mode {
        Mode::List => {
            title = format!("{reference} — diff");
            app.diff.iter().map(|l| colorize(l)).collect()
        }
        Mode::Tree => {
            title = format!("{reference} — files");
            build_tree(&app.files, &app.file_stats)
                .into_iter()
                .map(|row| match row {
                    TreeRow::Dir(s) => Line::styled(s, Style::default().fg(Color::DarkGray)),
                    TreeRow::File(i, s) => {
                        let style = if i == app.file_i {
                            Style::default().add_modifier(Modifier::REVERSED)
                        } else {
                            Style::default()
                        };
                        Line::styled(s, style)
                    }
                })
                .collect()
        }
        Mode::FileDiff => {
            title = app
                .files
                .get(app.file_i)
                .map(|p| format!("{p}  (h: back)"))
                .unwrap_or_default();
            app.diff.iter().map(|l| colorize(l)).collect()
        }
    };
    f.render_widget(
        Paragraph::new(lines)
            .block(Block::bordered().title(title))
            .scroll((app.scroll, 0)),
        right,
    );

    let keys = match app.mode {
        Mode::List => "j/k move  l files  Enter pop stash  Ctrl-u/d scroll  q quit",
        Mode::Tree => {
            "j/k file  l open  p pop file  h back  Enter pop stash  Ctrl-u/d scroll  q quit"
        }
        Mode::FileDiff => "j/k/Ctrl-u/d scroll  p pop file  h back  Enter pop stash  q quit",
    };
    let footer_text = match &app.status {
        Some(s) => format!("{keys}  |  {s}"),
        None => keys.to_string(),
    };
    f.render_widget(
        Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray)),
        footer,
    );

    if let Some(confirm) = app.confirm {
        let message = match confirm {
            Confirm::PopStash => format!("Pop {reference}? (y/n)"),
            Confirm::PopFile => {
                let file = app.files.get(app.file_i).map(String::as_str).unwrap_or("?");
                format!("Pop only {file} from {reference}? (y/n)")
            }
        };
        let area = centered(main, message.len() as u16 + 4, 3);
        f.render_widget(Clear, area);
        f.render_widget(
            Paragraph::new(message).block(Block::bordered().title("confirm")),
            area,
        );
    }
}

fn colorize(line: &str) -> Line<'static> {
    let style = if line.starts_with("+++")
        || line.starts_with("---")
        || line.starts_with("diff ")
        || line.starts_with("index ")
    {
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD)
    } else if line.starts_with('+') {
        Style::default().fg(Color::Green)
    } else if line.starts_with('-') {
        Style::default().fg(Color::Red)
    } else if line.starts_with("@@") {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    Line::styled(line.to_string(), style)
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}
