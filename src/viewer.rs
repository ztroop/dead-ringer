use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;

pub struct FileDiffViewer {
    diffs: Vec<(usize, u8)>,
    cursor_pos: usize,
    scroll: usize,
    bytes_per_line: usize,
}

impl FileDiffViewer {
    pub fn new(diffs: Vec<(usize, u8)>) -> Self {
        Self {
            diffs,
            cursor_pos: 0,
            scroll: 0,
            bytes_per_line: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            Clear(ClearType::All),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture,
        )?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        'mainloop: loop {
            terminal.draw(|f| self.draw(f))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break 'mainloop,
                    KeyCode::Down | KeyCode::Char('j') => self.move_cursor_down(&terminal.size()?),
                    KeyCode::Up | KeyCode::Char('k') => self.move_cursor_up(),
                    KeyCode::Right | KeyCode::Char('l') => self.move_cursor_right(&terminal.size()?),
                    KeyCode::Left | KeyCode::Char('h') => self.move_cursor_left(),
                    _ => {}
                }
            }
        }

        // Cleanup
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture,
            Clear(ClearType::All)
        )?;
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame) {
        let size = f.size();

        let hex_section_width = (size.width as f32 * 0.7).floor() as usize;
        let padding_and_borders = 4;
        let adjusted_width = hex_section_width - padding_and_borders;
        self.bytes_per_line = adjusted_width / 3;

        let hex_width = (self.bytes_per_line * 3 + 2) as u16;
        let ascii_width = (self.bytes_per_line + 2) as u16;

        let lines = (size.height - 3) as usize;

        let hex_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(size.height.saturating_sub(3)),
                Constraint::Length(3),
            ])
            .split(size);

        let hex_ascii_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(hex_width),   // Hex view
                Constraint::Length(ascii_width), // ASCII view
            ])
            .split(hex_chunks[0]);

        // Prepare hex and ASCII lines
        let hex_lines = self
            .diffs
            .chunks(self.bytes_per_line)
            .skip(self.scroll)
            .take(lines)
            .enumerate()
            .map(|(line_idx, chunk)| {
                let spans: Vec<Span> = chunk
                    .iter()
                    .enumerate()
                    .map(|(idx, &(_, byte))| {
                        let pos = (line_idx + self.scroll) * self.bytes_per_line + idx;
                        let style = if pos == self.cursor_pos {
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::REVERSED)
                        } else {
                            byte_style(byte)
                        };
                        Span::styled(format!("{:02x} ", byte), style)
                    })
                    .collect();
                Line::from(spans)
            })
            .collect::<Vec<_>>();

        let ascii_lines = self
            .diffs
            .chunks(self.bytes_per_line)
            .skip(self.scroll)
            .take(lines)
            .enumerate()
            .map(|(line_idx, chunk)| {
                let spans: Vec<Span> = chunk
                    .iter()
                    .enumerate()
                    .map(|(idx, &(_, byte))| {
                        let pos = (line_idx + self.scroll) * self.bytes_per_line + idx;
                        let style = if pos == self.cursor_pos {
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::REVERSED)
                        } else {
                            byte_style(byte)
                        };
                        let ascii_char = if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
                            byte as char
                        } else {
                            '.'
                        };
                        Span::styled(ascii_char.to_string(), style)
                    })
                    .collect();
                Line::from(spans)
            })
            .collect::<Vec<_>>();

        let hex_paragraph =
            Paragraph::new(hex_lines).block(Block::default().borders(Borders::ALL).title("Hex"));
        let ascii_paragraph = Paragraph::new(ascii_lines)
            .block(Block::default().borders(Borders::ALL).title("ASCII"));

        f.render_widget(hex_paragraph, hex_ascii_chunks[0]);
        f.render_widget(ascii_paragraph, hex_ascii_chunks[1]);

        // Info bar
        if self.cursor_pos < self.diffs.len() {
            let offset = self.diffs[self.cursor_pos].0;
            let info_text = Text::from(Span::from(format!("Position: {:08x}", offset)));
            let info_paragraph = Paragraph::new(info_text)
                .block(Block::default().borders(Borders::ALL).title("Info"));
            f.render_widget(info_paragraph, hex_chunks[1]);
        }
    }

    fn move_cursor_down(&mut self, terminal_size: &Rect) {
        let lines = (terminal_size.height - 5) as usize;
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

    fn move_cursor_up(&mut self) {
        if self.cursor_pos >= self.bytes_per_line {
            self.cursor_pos = self.cursor_pos.saturating_sub(self.bytes_per_line);
        }

        // Adjust scrolling if cursor moves above the visible area
        if self.cursor_pos / self.bytes_per_line < self.scroll {
            self.scroll = self.scroll.saturating_sub(1);
        }
    }

    fn move_cursor_right(&mut self, terminal_size: &Rect) {
        let lines = (terminal_size.height - 5) as usize;
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

    fn move_cursor_left(&mut self) {
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
}

pub fn byte_style(byte: u8) -> Style {
    if byte == 0 {
        Style::default().fg(Color::Gray)
    } else if byte.is_ascii_graphic() {
        Style::default().fg(Color::Cyan)
    } else if byte.is_ascii_whitespace() || byte.is_ascii() {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Yellow)
    }
}
