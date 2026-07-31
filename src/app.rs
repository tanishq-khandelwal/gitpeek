use crate::git::{self, Stash};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    List,
    Tree,
    FileDiff,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Confirm {
    PopStash,
    PopFile,
}

pub struct App {
    pub stashes: Vec<Stash>,
    pub list_i: usize,
    pub mode: Mode,
    /// Confirm prompt is separate from `mode` so cancelling needs no saved mode.
    pub confirm: Option<Confirm>,
    pub files: Vec<String>,
    /// Parallel to `files`: (lines added, lines deleted), from `git diff --numstat`.
    pub file_stats: Vec<(u32, u32)>,
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
            confirm: None,
            files: Vec::new(),
            file_stats: Vec::new(),
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
        let result = match self.mode {
            Mode::List => git::diff(&reference),
            Mode::Tree => {
                self.diff.clear();
                return;
            }
            Mode::FileDiff => {
                let Some(path) = self.files.get(self.file_i) else {
                    self.diff.clear();
                    return;
                };
                git::file_diff(&reference, path)
            }
        };
        match result {
            Ok(text) => self.diff = text.lines().map(str::to_string).collect(),
            Err(e) => {
                self.diff.clear();
                self.status = Some(e.to_string());
            }
        }
    }

    /// Moves the selection within the current mode's list (stashes, or files in the tree).
    pub fn select(&mut self, delta: isize) {
        let (i, len) = match self.mode {
            Mode::List => (&mut self.list_i, self.stashes.len()),
            Mode::Tree => (&mut self.file_i, self.files.len()),
            Mode::FileDiff => return,
        };
        if len == 0 {
            return;
        }
        *i = (*i as isize + delta).clamp(0, len as isize - 1) as usize;
        if self.mode == Mode::List {
            self.refresh_diff();
        }
    }

    /// Moves the stash selection directly regardless of mode - used by mouse-wheel
    /// scrolling over the stash list, which stays visible no matter how deep you are.
    pub fn select_stash(&mut self, delta: isize) {
        if self.stashes.is_empty() {
            return;
        }
        self.list_i =
            (self.list_i as isize + delta).clamp(0, self.stashes.len() as isize - 1) as usize;
        self.back_to_list();
    }

    pub fn scroll_by(&mut self, delta: i32) {
        let max = match self.mode {
            // ponytail: rough upper bound (files + a directory header per file) rather
            // than the exact rendered row count, to avoid app.rs depending on ui.rs.
            Mode::Tree => (self.files.len() * 2) as i32,
            _ => self.diff.len().saturating_sub(1) as i32,
        };
        self.scroll = (self.scroll as i32 + delta).clamp(0, max.max(0)) as u16;
    }

    pub fn enter_tree(&mut self) {
        let Some(reference) = self.current().map(|s| s.reference.clone()) else {
            return;
        };
        match git::file_stats(&reference) {
            Ok(stats) if stats.is_empty() => self.status = Some("no files in this stash".into()),
            Ok(stats) => {
                self.files = stats.iter().map(|(path, ..)| path.clone()).collect();
                self.file_stats = stats.into_iter().map(|(_, a, d)| (a, d)).collect();
                self.file_i = 0;
                self.mode = Mode::Tree;
                self.status = None;
                self.diff.clear();
                self.scroll = 0;
            }
            Err(e) => self.status = Some(e.to_string()),
        }
    }

    pub fn enter_file_diff(&mut self) {
        if self.files.is_empty() {
            return;
        }
        self.mode = Mode::FileDiff;
        self.refresh_diff();
    }

    pub fn back_to_tree(&mut self) {
        self.mode = Mode::Tree;
        self.diff.clear();
        self.scroll = 0;
    }

    pub fn back_to_list(&mut self) {
        self.mode = Mode::List;
        self.files.clear();
        self.file_stats.clear();
        self.file_i = 0;
        self.refresh_diff();
    }

    pub fn do_pop_stash(&mut self) {
        self.confirm = None;
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

    /// Pops just the selected file out of the current stash, leaving the rest stashed.
    pub fn do_pop_file(&mut self) {
        self.confirm = None;
        let Some(reference) = self.current().map(|s| s.reference.clone()) else {
            return;
        };
        let Some(path) = self.files.get(self.file_i).cloned() else {
            return;
        };
        match git::pop_file(&reference, &path, &self.files) {
            Ok(text) => {
                self.popped = Some(format!(
                    "Popped {path} from {reference}\n{}",
                    text.trim_end()
                ));
                self.should_quit = true;
            }
            Err(e) => self.status = Some(e.to_string()),
        }
    }
}
