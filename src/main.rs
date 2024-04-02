use std::io;

use app::App;
use event::{Event, EventHandler};
use file::{diff_files, read_file};
use handler::handle_key_events;
use ratatui::{backend::CrosstermBackend, Terminal};
use tui::Tui;

mod app;
mod event;
mod file;
mod handler;
mod tui;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <file1> <file2>", args[0]);
        std::process::exit(1);
    }

    let file1_data = read_file(&args[1])?;
    let file2_data = read_file(&args[2])?;
    let diffs = diff_files(&file1_data, &file2_data);

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(1_000);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    let mut app = App::new(file1_data, file2_data, diffs);
    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Tick => app.tick()?,
            Event::Key(key_event) => handle_key_events(key_event, &mut app, tui.size())?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;
    Ok(())
}
