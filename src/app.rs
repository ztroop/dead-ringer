use std::error;

use crate::clipboard::{format_ascii, format_hex, osc52_copy, Selection};
use crate::search::{SearchKind, SearchState};

/// Lines reserved for info bar and borders; visible content lines = height - VISIBLE_LINES_OFFSET.
pub const VISIBLE_LINES_OFFSET: u16 = 5;

/// Transient feedback message shown in the info bar after an action.
#[derive(Debug, Clone)]
pub struct Flash {
    pub message: String,
    pub ticks_remaining: u8,
}

pub struct App {
    pub running: bool,
    pub diffs: Vec<(usize, u8)>,
    pub cursor_pos: usize,
    pub scroll: usize,
    pub bytes_per_line: usize,
    pub search: SearchState,
    pub selection: Option<Selection>,
    pub flash: Option<Flash>,
}

impl App {
    pub fn new(diffs: Vec<(usize, u8)>) -> Self {
        Self {
            running: true,
            diffs,
            cursor_pos: 0,
            scroll: 0,
            bytes_per_line: 0,
            search: SearchState::default(),
            selection: None,
            flash: None,
        }
    }

    pub fn tick(&mut self) -> Result<(), Box<dyn error::Error>> {
        if let Some(ref mut f) = self.flash {
            f.ticks_remaining = f.ticks_remaining.saturating_sub(1);
            if f.ticks_remaining == 0 {
                self.flash = None;
            }
        }
        Ok(())
    }

    pub fn move_cursor_down(&mut self, terminal_height: u16) {
        let lines = terminal_height.saturating_sub(VISIBLE_LINES_OFFSET) as usize;
        let max_cursor_pos = self.diffs.len().saturating_sub(1);

        if self.cursor_pos < max_cursor_pos {
            self.cursor_pos += self.bytes_per_line;
            self.cursor_pos = self.cursor_pos.min(max_cursor_pos);
        }

        if (self.cursor_pos / self.bytes_per_line) >= (self.scroll + lines)
            && (self.scroll + lines) < self.diffs.len().div_ceil(self.bytes_per_line)
        {
            self.scroll += 1;
        }

        self.update_selection_cursor();
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_pos >= self.bytes_per_line {
            self.cursor_pos = self.cursor_pos.saturating_sub(self.bytes_per_line);
        }

        if self.cursor_pos / self.bytes_per_line < self.scroll {
            self.scroll = self.scroll.saturating_sub(1);
        }

        self.update_selection_cursor();
    }

    pub fn move_cursor_right(&mut self, terminal_height: u16) {
        let lines = terminal_height.saturating_sub(VISIBLE_LINES_OFFSET) as usize;
        let max_cursor_pos = self.diffs.len().saturating_sub(1);

        if self.cursor_pos < max_cursor_pos {
            self.cursor_pos += 1;
        }

        let cursor_line = self.cursor_pos / self.bytes_per_line;
        if cursor_line >= self.scroll + lines {
            self.scroll = cursor_line + 1 - lines;
        }

        self.update_selection_cursor();
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }

        let cursor_line = self.cursor_pos / self.bytes_per_line;
        if cursor_line < self.scroll {
            self.scroll = cursor_line;
        }

        self.update_selection_cursor();
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
            let visible_lines = terminal_height.saturating_sub(VISIBLE_LINES_OFFSET) as usize;
            let cursor_line = pos / self.bytes_per_line;
            if cursor_line < self.scroll || cursor_line >= self.scroll + visible_lines {
                self.scroll = cursor_line.saturating_sub(visible_lines / 2);
            }
        }
    }

    /// Enter visual selection mode, anchoring at the current cursor position.
    pub fn start_selection(&mut self) {
        self.selection = Some(Selection::new(self.cursor_pos));
    }

    /// Cancel visual selection mode.
    pub fn cancel_selection(&mut self) {
        self.selection = None;
    }

    /// Keep the selection's moving end in sync with the cursor.
    fn update_selection_cursor(&mut self) {
        if let Some(ref mut sel) = self.selection {
            sel.cursor = self.cursor_pos;
        }
    }

    /// Copy the selected bytes as hex to the clipboard via OSC 52, then exit visual mode.
    pub fn yank_hex(&mut self) {
        if let Some(text) = self.selected_as_hex() {
            let count = self.selection.map_or(0, |s| s.len());
            let _ = osc52_copy(&text);
            self.selection = None;
            self.flash = Some(Flash {
                message: format!("Copied {} byte(s) as hex", count),
                ticks_remaining: 3,
            });
        }
    }

    /// Copy the selected bytes as ASCII to the clipboard via OSC 52, then exit visual mode.
    pub fn yank_ascii(&mut self) {
        if let Some(text) = self.selected_as_ascii() {
            let count = self.selection.map_or(0, |s| s.len());
            let _ = osc52_copy(&text);
            self.selection = None;
            self.flash = Some(Flash {
                message: format!("Copied {} byte(s) as ASCII", count),
                ticks_remaining: 3,
            });
        }
    }

    /// Return the selected diff bytes as a hex string.
    fn selected_as_hex(&self) -> Option<String> {
        self.selected_bytes().map(|b| format_hex(&b))
    }

    /// Return the selected diff bytes as an ASCII string.
    fn selected_as_ascii(&self) -> Option<String> {
        self.selected_bytes().map(|b| format_ascii(&b))
    }

    /// Extract the raw byte values covered by the current selection.
    fn selected_bytes(&self) -> Option<Vec<u8>> {
        let sel = self.selection?;
        let end = sel.end();
        if end >= self.diffs.len() {
            return None;
        }
        Some(
            self.diffs[sel.start()..=end]
                .iter()
                .map(|&(_, b)| b)
                .collect(),
        )
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
