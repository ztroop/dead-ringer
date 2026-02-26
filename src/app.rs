use std::error;

use crate::search::{SearchKind, SearchState};

pub struct App {
    pub running: bool,
    pub file1_data: Vec<u8>,
    pub file2_data: Vec<u8>,
    pub diffs: Vec<(usize, u8)>,
    pub cursor_pos: usize,
    pub scroll: usize,
    pub bytes_per_line: usize,
    pub search: SearchState,
}

impl App {
    pub fn new(file1_data: Vec<u8>, file2_data: Vec<u8>, diffs: Vec<(usize, u8)>) -> Self {
        Self {
            running: true,
            file1_data,
            file2_data,
            diffs,
            cursor_pos: 0,
            scroll: 0,
            bytes_per_line: 0,
            search: SearchState::default(),
        }
    }

    pub fn tick(&mut self) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    pub fn move_cursor_down(&mut self, terminal_height: u16) {
        let lines = (terminal_height - 5) as usize;
        let max_cursor_pos = self.diffs.len().saturating_sub(1);

        if self.cursor_pos < max_cursor_pos {
            self.cursor_pos += self.bytes_per_line;
            self.cursor_pos = self.cursor_pos.min(max_cursor_pos);
        }

        if (self.cursor_pos / self.bytes_per_line) >= (self.scroll + lines)
            && (self.scroll + lines)
                < ((self.diffs.len() + self.bytes_per_line - 1) / self.bytes_per_line)
        {
            self.scroll += 1;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_pos >= self.bytes_per_line {
            self.cursor_pos = self.cursor_pos.saturating_sub(self.bytes_per_line);
        }

        if self.cursor_pos / self.bytes_per_line < self.scroll {
            self.scroll = self.scroll.saturating_sub(1);
        }
    }

    pub fn move_cursor_right(&mut self, terminal_height: u16) {
        let lines = (terminal_height - 5) as usize;
        let max_cursor_pos = self.diffs.len().saturating_sub(1);

        if self.cursor_pos < max_cursor_pos {
            self.cursor_pos += 1;
        }

        let cursor_line = self.cursor_pos / self.bytes_per_line;
        if cursor_line >= self.scroll + lines {
            self.scroll = cursor_line + 1 - lines;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }

        let cursor_line = self.cursor_pos / self.bytes_per_line;
        if cursor_line < self.scroll {
            self.scroll = cursor_line;
        }
    }

    /// Begin a new search in the given mode (hex or ASCII).
    pub fn start_search(&mut self, kind: SearchKind) {
        self.search.start(kind);
    }

    /// Execute the current search query and jump to the first match.
    pub fn submit_search(&mut self, terminal_height: u16) {
        self.search.submit(&self.diffs);
        self.navigate_to_current_match(terminal_height);
    }

    /// Jump to the next search match, wrapping around.
    pub fn next_match(&mut self, terminal_height: u16) {
        self.search.next_match();
        self.navigate_to_current_match(terminal_height);
    }

    /// Jump to the previous search match, wrapping around.
    pub fn prev_match(&mut self, terminal_height: u16) {
        self.search.prev_match();
        self.navigate_to_current_match(terminal_height);
    }

    /// Move the cursor and scroll to center the current match on screen.
    fn navigate_to_current_match(&mut self, terminal_height: u16) {
        if self.bytes_per_line == 0 {
            return;
        }
        if let Some(pos) = self.search.current_match_pos() {
            self.cursor_pos = pos;
            let visible_lines = (terminal_height.saturating_sub(5)) as usize;
            let cursor_line = pos / self.bytes_per_line;
            if cursor_line < self.scroll || cursor_line >= self.scroll + visible_lines {
                self.scroll = cursor_line.saturating_sub(visible_lines / 2);
            }
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
