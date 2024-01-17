use clap::{Arg, command};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::fs::File;
use std::io::{self, Read};
use tui::{backend::CrosstermBackend, Terminal};

mod display;

fn main() -> Result<(), io::Error> {
    let matches = command!()
        .arg(Arg::new("file1")
            .help("Path to the first binary file")
            .required(true)
            .index(1))
        .arg(Arg::new("file2")
            .help("Path to the second binary file")
            .required(true)
            .index(2))
        .get_matches();

    let file1_path = matches.get_one::<String>("file1").unwrap();
    let file2_path = matches.get_one::<String>("file2").unwrap();

    let contents1 = read_file(file1_path)?;
    let contents2 = read_file(file2_path)?;

    let diff = compare_files(&contents1, &contents2);

    display_diff(diff.0, diff.1)?;

    Ok(())
}

fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

fn compare_files(file1: &[u8], file2: &[u8]) -> (String, String) {
    let len = std::cmp::min(file1.len(), file2.len());
    let mut hex_diff = String::new();
    let mut ascii_diff = String::new();

    for i in 0..len {
        if file1[i] != file2[i] {
            hex_diff.push_str(&format!("{:02x} {:02x} ", file1[i], file2[i]));
            ascii_diff.push_str(&format!("{} {} ", to_ascii(file1[i]), to_ascii(file2[i])));
        }
    }

    if file1.len() != file2.len() {
        hex_diff.push_str(&format!(
            "Different lengths: {} vs {}",
            file1.len(),
            file2.len()
        ));
        ascii_diff.push_str(" ");
    }

    (hex_diff, ascii_diff)
}

fn to_ascii(byte: u8) -> char {
    if byte.is_ascii_graphic() || byte == b' ' {
        byte as char
    } else {
        '.'
    }
}

fn display_diff(hex_diff: String, ascii_diff: String) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = display::TerminalApp::new(hex_diff, ascii_diff);

    loop {
        terminal.draw(|f| app.draw(f))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Down => app.update_scroll(display::ScrollDirection::Down),
                KeyCode::Up => app.update_scroll(display::ScrollDirection::Up),
                _ => {}
            }
        }
    }

    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()
}
