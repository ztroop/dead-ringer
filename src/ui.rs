use std::collections::HashSet;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, VISIBLE_LINES_OFFSET};
use crate::clipboard::Selection;
use crate::search::{SearchKind, SearchMode};

/// Render the user interface.
pub fn render(app: &mut App, frame: &mut Frame) {
    let size = frame.area();

    let hex_section_width = (size.width as f32 * 0.7).floor() as usize;
    let padding_and_borders = 4;
    let adjusted_width = hex_section_width.saturating_sub(padding_and_borders);
    app.bytes_per_line = (adjusted_width / 3).max(1);

    let hex_width = (app.bytes_per_line * 3 + 2) as u16;
    let ascii_width = (app.bytes_per_line + 2) as u16;
    let lines = size.height.saturating_sub(VISIBLE_LINES_OFFSET) as usize;

    let total_lines = app.diffs.len().div_ceil(app.bytes_per_line);
    let max_scroll = total_lines.saturating_sub(lines);
    app.scroll = app.scroll.min(max_scroll);

    let hex_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(size.height.saturating_sub(3)),
            Constraint::Length(3),
        ])
        .split(size);

    let hex_ascii_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(hex_width), Constraint::Min(ascii_width)])
        .split(hex_chunks[0]);

    let all_matches = app.search.all_match_positions();
    let current_match = app.search.current_match_set();

    let ctx = StyleContext {
        cursor_pos: app.cursor_pos,
        all_matches: &all_matches,
        current_match: &current_match,
        selection: app.selection.as_ref(),
    };

    let hex_lines = build_hex_lines(app, lines, &ctx);
    let ascii_lines = build_ascii_lines(app, lines, &ctx);

    let hex_paragraph =
        Paragraph::new(hex_lines).block(Block::default().borders(Borders::ALL).title("Hex"));
    let ascii_paragraph =
        Paragraph::new(ascii_lines).block(Block::default().borders(Borders::ALL).title("ASCII"));

    frame.render_widget(hex_paragraph, hex_ascii_chunks[0]);
    frame.render_widget(ascii_paragraph, hex_ascii_chunks[1]);

    render_info_bar(app, frame, hex_chunks[1]);
}

/// Aggregates all the state needed to pick a style for any byte position.
struct StyleContext<'a> {
    cursor_pos: usize,
    all_matches: &'a HashSet<usize>,
    current_match: &'a HashSet<usize>,
    selection: Option<&'a Selection>,
}

fn build_hex_lines<'a>(app: &App, visible_lines: usize, ctx: &StyleContext) -> Vec<Line<'a>> {
    app.diffs
        .chunks(app.bytes_per_line)
        .skip(app.scroll)
        .take(visible_lines)
        .enumerate()
        .map(|(line_idx, chunk)| {
            let spans: Vec<Span> = chunk
                .iter()
                .enumerate()
                .map(|(idx, &(_, byte))| {
                    let pos = (line_idx + app.scroll) * app.bytes_per_line + idx;
                    let style = pos_style(pos, byte, ctx);
                    Span::styled(format!("{:02x} ", byte), style)
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}

fn build_ascii_lines<'a>(app: &App, visible_lines: usize, ctx: &StyleContext) -> Vec<Line<'a>> {
    app.diffs
        .chunks(app.bytes_per_line)
        .skip(app.scroll)
        .take(visible_lines)
        .enumerate()
        .map(|(line_idx, chunk)| {
            let spans: Vec<Span> = chunk
                .iter()
                .enumerate()
                .map(|(idx, &(_, byte))| {
                    let pos = (line_idx + app.scroll) * app.bytes_per_line + idx;
                    let style = pos_style(pos, byte, ctx);
                    let ch = if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
                        byte as char
                    } else {
                        '.'
                    };
                    Span::styled(ch.to_string(), style)
                })
                .collect();
            Line::from(spans)
        })
        .collect()
}

/// Determine the display style for a byte at a given position.
///
/// Priority (highest to lowest):
/// cursor > selection > current match > any match > default.
fn pos_style(pos: usize, byte: u8, ctx: &StyleContext) -> Style {
    if pos == ctx.cursor_pos {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::REVERSED)
    } else if ctx.selection.is_some_and(|sel| sel.contains(pos)) {
        Style::default().fg(Color::Black).bg(Color::LightBlue)
    } else if ctx.current_match.contains(&pos) {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if ctx.all_matches.contains(&pos) {
        Style::default().fg(Color::Black).bg(Color::DarkGray)
    } else {
        byte_style(byte)
    }
}

/// Render the bottom info/search bar.
fn render_info_bar(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    match app.search.mode {
        SearchMode::Input(kind) => {
            let label = match kind {
                SearchKind::Hex => "hex",
                SearchKind::Ascii => "ascii",
            };
            let text = Text::from(Span::from(format!(
                "Search ({}): {}█",
                label, app.search.query
            )));
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Search [Tab to toggle mode]");
            frame.render_widget(Paragraph::new(text).block(block), area);
        }
        SearchMode::Normal => {
            let mut parts: Vec<Span> = Vec::new();

            if app.diffs.is_empty() && app.selection.is_none() {
                parts.push(Span::styled(
                    "Files are identical".to_string(),
                    Style::default().fg(Color::Green),
                ));
            } else if app.cursor_pos < app.diffs.len() {
                let offset = app.diffs[app.cursor_pos].0;
                parts.push(Span::from(format!("Position: {:08x}", offset)));
            }

            if let Some(ref sel) = app.selection {
                if !parts.is_empty() {
                    parts.push(Span::from("  │  "));
                }
                let count = if sel.end() >= app.diffs.len() {
                    0
                } else {
                    sel.len()
                };
                parts.push(Span::styled(
                    format!("VISUAL {} byte(s)", count),
                    Style::default()
                        .fg(Color::LightBlue)
                        .add_modifier(Modifier::BOLD),
                ));
                parts.push(Span::styled(
                    "  y hex  Y ascii  Esc cancel".to_string(),
                    Style::default().fg(Color::DarkGray),
                ));
            } else if let Some(ref flash) = app.flash {
                if !parts.is_empty() {
                    parts.push(Span::from("  │  "));
                }
                parts.push(Span::styled(
                    flash.message.clone(),
                    Style::default().fg(Color::Green),
                ));
            } else if let Some(idx) = app.search.current_match {
                if !parts.is_empty() {
                    parts.push(Span::from("  │  "));
                }
                let kind_label = match app.search.kind {
                    SearchKind::Hex => "hex",
                    SearchKind::Ascii => "ascii",
                };
                parts.push(Span::styled(
                    format!(
                        "Match {}/{} ({}: {})",
                        idx + 1,
                        app.search.matches.len(),
                        kind_label,
                        app.search.query,
                    ),
                    Style::default().fg(Color::Yellow),
                ));
                parts.push(Span::styled(
                    "  n/N next/prev".to_string(),
                    Style::default().fg(Color::DarkGray),
                ));
            } else if !app.search.query.is_empty() && app.search.matches.is_empty() {
                if !parts.is_empty() {
                    parts.push(Span::from("  │  "));
                }
                parts.push(Span::styled(
                    "No matches".to_string(),
                    Style::default().fg(Color::Red),
                ));
            } else {
                if !parts.is_empty() {
                    parts.push(Span::from("  │  "));
                }
                parts.push(Span::styled(
                    "/ hex search  ? ascii search  v select  q quit".to_string(),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            let text = Text::from(Line::from(parts));
            let paragraph =
                Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Info"));
            frame.render_widget(paragraph, area);
        }
    }
}

/// Color-code a byte value for the default (non-highlighted) display.
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
