use std::io::Stdout;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};

pub struct TerminalApp {
    pub hex_diff: String,
    pub ascii_diff: String,
    pub hex_scroll: usize,
    pub ascii_scroll: usize,
}

impl TerminalApp {
    pub fn new(hex_diff: String, ascii_diff: String) -> TerminalApp {
        TerminalApp {
            hex_diff,
            ascii_diff,
            hex_scroll: 0,
            ascii_scroll: 0,
        }
    }

    pub fn draw(&mut self, f: &mut tui::Frame<CrosstermBackend<Stdout>>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        let hex_diff_paragraph = Paragraph::new(self.hex_diff.clone())
            .block(Block::default().borders(Borders::ALL).title("Hex Diff"))
            .wrap(tui::widgets::Wrap { trim: true })
            .scroll((self.hex_scroll as u16, 0));
        f.render_widget(hex_diff_paragraph, chunks[0]);

        let ascii_diff_paragraph = Paragraph::new(self.ascii_diff.clone())
            .block(Block::default().borders(Borders::ALL).title("ASCII Diff"))
            .wrap(tui::widgets::Wrap { trim: true })
            .scroll((self.ascii_scroll as u16, 0));
        f.render_widget(ascii_diff_paragraph, chunks[1]);
    }

    pub fn update_scroll(&mut self, direction: ScrollDirection) {
        match direction {
            ScrollDirection::Up => {
                self.hex_scroll = self.hex_scroll.saturating_sub(1);
                self.ascii_scroll = self.ascii_scroll.saturating_sub(1);
            }
            ScrollDirection::Down => {
                self.hex_scroll = self.hex_scroll.saturating_add(1);
                self.ascii_scroll = self.ascii_scroll.saturating_add(1);
            }
        }
    }
}

pub enum ScrollDirection {
    Up,
    Down,
}
