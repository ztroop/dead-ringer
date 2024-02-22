use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <file1> <file2>", args[0]);
        std::process::exit(1);
    }

    let file1_data = read_file(&args[1])?;
    let file2_data = read_file(&args[2])?;
    let diffs = diff_files(&file1_data, &file2_data);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        Clear(ClearType::All)
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut cursor_pos: usize = 0;
    let mut scroll: usize = 0; // To keep track of the vertical scroll

    'mainloop: loop {
        terminal.draw(|f| {
            let size = f.size();
            let bytes_per_line = (size.width / 6) as usize; // Adjusted for 3 chars per byte and space for ASCII
            let lines = (size.height - 3) as usize; // Subtracting height for info bar and potential borders

            let hex_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(size.height.saturating_sub(3)), // Hex and ASCII view
                    Constraint::Length(3),                          // Info bar
                ])
                .split(size);

            let hex_ascii_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(70), // Hex view
                    Constraint::Percentage(30), // ASCII view
                ])
                .split(hex_chunks[0]);

            // Prepare hex and ASCII lines
            let hex_lines = diffs
                .chunks(bytes_per_line)
                .skip(scroll)
                .take(lines)
                .enumerate()
                .map(|(line_idx, chunk)| {
                    let spans: Vec<Span> = chunk
                        .iter()
                        .enumerate()
                        .map(|(idx, &(_, byte))| {
                            let pos = (line_idx + scroll) * bytes_per_line + idx;
                            let style = if pos == cursor_pos {
                                Style::default()
                                    .fg(Color::White)
                                    .add_modifier(Modifier::REVERSED)
                            } else {
                                byte_style(byte)
                            };
                            Span::styled(format!("{:02x} ", byte), style)
                        })
                        .collect();
                    Spans::from(spans)
                })
                .collect::<Vec<_>>();

            let ascii_lines = diffs
                .chunks(bytes_per_line)
                .skip(scroll)
                .take(lines)
                .enumerate()
                .map(|(line_idx, chunk)| {
                    let spans: Vec<Span> = chunk
                        .iter()
                        .enumerate()
                        .map(|(idx, &(_, byte))| {
                            let pos = (line_idx + scroll) * bytes_per_line + idx;
                            let style = if pos == cursor_pos {
                                Style::default()
                                    .fg(Color::White)
                                    .add_modifier(Modifier::REVERSED)
                            } else {
                                byte_style(byte)
                            };
                            let ascii_char =
                                if byte.is_ascii_graphic() || byte.is_ascii_whitespace() {
                                    byte as char
                                } else {
                                    '.'
                                };
                            Span::styled(ascii_char.to_string(), style)
                        })
                        .collect();
                    Spans::from(spans)
                })
                .collect::<Vec<_>>();

            let hex_paragraph = Paragraph::new(hex_lines)
                .block(Block::default().borders(Borders::ALL).title("Hex"));
            let ascii_paragraph = Paragraph::new(ascii_lines)
                .block(Block::default().borders(Borders::ALL).title("ASCII"));

            f.render_widget(hex_paragraph, hex_ascii_chunks[0]);
            f.render_widget(ascii_paragraph, hex_ascii_chunks[1]);

            // Info bar
            if cursor_pos < diffs.len() {
                let (offset, _) = diffs[cursor_pos];
                let info_text = Text::from(Span::from(format!("Position: {:08x}", offset)));
                let info_paragraph = Paragraph::new(info_text)
                    .block(Block::default().borders(Borders::ALL).title("Info"));
                f.render_widget(info_paragraph, hex_chunks[1]);
            }
        })?;

        let size = terminal.size()?;
        let bytes_per_line = (size.width / 6) as usize; // For navigation logic
        let lines = (size.height - 3) as usize; // For scroll logic

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break 'mainloop,
                KeyCode::Down => {
                    if cursor_pos + bytes_per_line < diffs.len() {
                        cursor_pos += bytes_per_line;
                        if cursor_pos / bytes_per_line >= scroll + lines {
                            scroll += 1;
                        }
                    }
                }
                KeyCode::Up => {
                    if cursor_pos >= bytes_per_line {
                        cursor_pos -= bytes_per_line;
                        if cursor_pos / bytes_per_line < scroll {
                            scroll = scroll.saturating_sub(1);
                        }
                    }
                }
                KeyCode::Right => {
                    if cursor_pos < diffs.len() - 1 {
                        cursor_pos += 1;
                        if cursor_pos / bytes_per_line >= scroll + lines {
                            scroll += 1;
                        }
                    }
                }
                KeyCode::Left => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                        if cursor_pos / bytes_per_line < scroll {
                            scroll = scroll.saturating_sub(1);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        Clear(ClearType::All)
    )?;
    Ok(())
}

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn diff_files(file1: &[u8], file2: &[u8]) -> Vec<(usize, u8)> {
    file1
        .iter()
        .zip(file2.iter())
        .enumerate()
        .filter_map(|(i, (&b1, &b2))| if b1 != b2 { Some((i, b1)) } else { None })
        .collect()
}

fn byte_style(byte: u8) -> Style {
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
