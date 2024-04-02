use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    let size = frame.size();

    let hex_section_width = (size.width as f32 * 0.7).floor() as usize;
    let padding_and_borders = 4;
    let adjusted_width = hex_section_width - padding_and_borders;
    app.bytes_per_line = adjusted_width / 3;

    let hex_width = (app.bytes_per_line * 3 + 2) as u16;
    let ascii_width = (app.bytes_per_line + 2) as u16;

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
    let hex_lines = app
        .diffs
        .chunks(app.bytes_per_line)
        .skip(app.scroll)
        .take(lines)
        .enumerate()
        .map(|(line_idx, chunk)| {
            let spans: Vec<Span> = chunk
                .iter()
                .enumerate()
                .map(|(idx, &(_, byte))| {
                    let pos = (line_idx + app.scroll) * app.bytes_per_line + idx;
                    let style = if pos == app.cursor_pos {
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

    let ascii_lines = app
        .diffs
        .chunks(app.bytes_per_line)
        .skip(app.scroll)
        .take(lines)
        .enumerate()
        .map(|(line_idx, chunk)| {
            let spans: Vec<Span> = chunk
                .iter()
                .enumerate()
                .map(|(idx, &(_, byte))| {
                    let pos = (line_idx + app.scroll) * app.bytes_per_line + idx;
                    let style = if pos == app.cursor_pos {
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
    let ascii_paragraph =
        Paragraph::new(ascii_lines).block(Block::default().borders(Borders::ALL).title("ASCII"));

    frame.render_widget(hex_paragraph, hex_ascii_chunks[0]);
    frame.render_widget(ascii_paragraph, hex_ascii_chunks[1]);

    // Info bar
    if app.cursor_pos < app.diffs.len() {
        let offset = app.diffs[app.cursor_pos].0;
        let info_text = Text::from(Span::from(format!("Position: {:08x}", offset)));
        let info_paragraph =
            Paragraph::new(info_text).block(Block::default().borders(Borders::ALL).title("Info"));
        frame.render_widget(info_paragraph, hex_chunks[1]);
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
