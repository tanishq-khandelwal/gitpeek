use crate::app::{App, Mode};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, List, ListState, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    let [main, footer] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(f.area());
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(main);

    let reference = app.current().map(|s| s.reference.as_str()).unwrap_or("-");
    let (title, items, selected) = match app.mode {
        Mode::List => (
            "git stashes".to_string(),
            app.stashes
                .iter()
                .map(|s| {
                    if s.branch.is_empty() {
                        format!("{}  {}", s.reference, s.message)
                    } else {
                        format!("{}  [{}] {}", s.reference, s.branch, s.message)
                    }
                })
                .collect::<Vec<_>>(),
            app.list_i,
        ),
        Mode::Files => (
            format!("{reference} — files"),
            app.files.clone(),
            app.file_i,
        ),
    };
    let list = List::new(items)
        .block(Block::bordered().title(title))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_stateful_widget(
        list,
        left,
        &mut ListState::default().with_selected(Some(selected)),
    );

    let diff: Vec<Line> = app.diff.iter().map(|l| colorize(l)).collect();
    f.render_widget(
        Paragraph::new(diff)
            .block(Block::bordered().title("diff"))
            .scroll((app.scroll, 0)),
        right,
    );

    let keys = match app.mode {
        Mode::List => "j/k move  l files  Enter pop  Ctrl-u/d scroll  q quit",
        Mode::Files => "j/k file  h back  Enter pop  Ctrl-u/d scroll  q quit",
    };
    let footer_text = match &app.status {
        Some(s) => format!("{keys}  |  {s}"),
        None => keys.to_string(),
    };
    f.render_widget(
        Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray)),
        footer,
    );

    if app.confirm {
        let area = centered(main, 40, 3);
        f.render_widget(Clear, area);
        f.render_widget(
            Paragraph::new(format!("Pop {reference}? (y/n)"))
                .block(Block::bordered().title("confirm")),
            area,
        );
    }
}

fn colorize(line: &str) -> Line<'_> {
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
    Line::styled(line, style)
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
