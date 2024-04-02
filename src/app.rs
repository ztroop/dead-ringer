use std::error;

pub struct App {
    pub running: bool,
    pub file1_data: Vec<u8>,
    pub file2_data: Vec<u8>,
    pub diffs: Vec<(usize, u8)>,
    pub cursor_pos: usize,
    pub scroll: usize,
    pub bytes_per_line: usize,
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
        }
    }

    pub fn tick(&mut self) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }

    pub fn move_cursor_down(&mut self, terminal_height: u16) {
        let lines = (terminal_height - 5) as usize;
        let max_cursor_pos = self.diffs.len().saturating_sub(1);

        // Increment cursor position if not at the end of diffs
        if self.cursor_pos < max_cursor_pos {
            self.cursor_pos += self.bytes_per_line;
            self.cursor_pos = self.cursor_pos.min(max_cursor_pos);
        }

        // Adjust scrolling if cursor moves beyond the visible area
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

        // Adjust scrolling if cursor moves above the visible area
        if self.cursor_pos / self.bytes_per_line < self.scroll {
            self.scroll = self.scroll.saturating_sub(1);
        }
    }

    pub fn move_cursor_right(&mut self, terminal_height: u16) {
        let lines = (terminal_height - 5) as usize;
        let max_cursor_pos = self.diffs.len().saturating_sub(1);

        // Move cursor right if not at the end of diffs
        if self.cursor_pos < max_cursor_pos {
            self.cursor_pos += 1;
        }

        // Special handling for bottom right movement
        let cursor_line = self.cursor_pos / self.bytes_per_line;
        if cursor_line >= self.scroll + lines {
            self.scroll = cursor_line + 1 - lines;
        }
    }

    pub fn move_cursor_left(&mut self) {
        // Move cursor left if not at the start
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }

        // Special handling for top left movement
        let cursor_line = self.cursor_pos / self.bytes_per_line;
        if cursor_line < self.scroll {
            self.scroll = cursor_line;
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
