use crate::git::{self, Stash};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    List,
    Files,
}

pub struct App {
    pub stashes: Vec<Stash>,
    pub list_i: usize,
    pub mode: Mode,
    /// Confirm-pop prompt is a flag rather than a mode, so cancelling needs no saved mode.
    pub confirm: bool,
    pub files: Vec<String>,
    pub file_i: usize,
    pub diff: Vec<String>,
    pub scroll: u16,
    pub status: Option<String>,
    pub should_quit: bool,
    /// Set after a successful pop; printed after the terminal is restored.
    pub popped: Option<String>,
}

impl App {
    pub fn new(stashes: Vec<Stash>) -> Self {
        let mut app = Self {
            stashes,
            list_i: 0,
            mode: Mode::List,
            confirm: false,
            files: Vec::new(),
            file_i: 0,
            diff: Vec::new(),
            scroll: 0,
            status: None,
            should_quit: false,
            popped: None,
        };
        app.refresh_diff();
        app
    }

    pub fn current(&self) -> Option<&Stash> {
        self.stashes.get(self.list_i)
    }

    pub fn refresh_diff(&mut self) {
        self.scroll = 0;
        let Some(reference) = self.current().map(|s| s.reference.clone()) else {
            self.diff.clear();
            return;
        };
        let result = match (self.mode, self.files.get(self.file_i)) {
            (Mode::Files, Some(path)) => git::file_diff(&reference, path),
            _ => git::diff(&reference),
        };
        match result {
            Ok(text) => self.diff = text.lines().map(str::to_string).collect(),
            Err(e) => {
                self.diff.clear();
                self.status = Some(e.to_string());
            }
        }
    }

    pub fn select(&mut self, delta: isize) {
        let (i, len) = match self.mode {
            Mode::List => (&mut self.list_i, self.stashes.len()),
            Mode::Files => (&mut self.file_i, self.files.len()),
        };
        if len == 0 {
            return;
        }
        *i = (*i as isize + delta).clamp(0, len as isize - 1) as usize;
        self.refresh_diff();
    }

    pub fn scroll_by(&mut self, delta: i32) {
        let max = self.diff.len().saturating_sub(1) as i32;
        self.scroll = (self.scroll as i32 + delta).clamp(0, max.max(0)) as u16;
    }

    pub fn enter_files(&mut self) {
        let Some(reference) = self.current().map(|s| s.reference.clone()) else {
            return;
        };
        match git::files(&reference) {
            Ok(files) if files.is_empty() => self.status = Some("no files in this stash".into()),
            Ok(files) => {
                self.files = files;
                self.file_i = 0;
                self.mode = Mode::Files;
                self.status = None;
                self.refresh_diff();
            }
            Err(e) => self.status = Some(e.to_string()),
        }
    }

    pub fn back_to_list(&mut self) {
        self.mode = Mode::List;
        self.files.clear();
        self.file_i = 0;
        self.refresh_diff();
    }

    pub fn do_pop(&mut self) {
        self.confirm = false;
        let Some(reference) = self.current().map(|s| s.reference.clone()) else {
            return;
        };
        match git::pop(&reference) {
            Ok(text) => {
                self.popped = Some(format!("Popped {reference}\n{}", text.trim_end()));
                self.should_quit = true;
            }
            Err(e) => self.status = Some(e.to_string()),
        }
    }
}
