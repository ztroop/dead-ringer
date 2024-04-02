use crossterm::event::{KeyCode, KeyEvent};

use crate::{app::App, tui::TerminalSize};

pub fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    size: TerminalSize,
) -> Result<(), Box<dyn std::error::Error>> {
    match key_event.code {
        KeyCode::Char('q') => {
            app.quit();
        }
        KeyCode::Down | KeyCode::Char('j') => app.move_cursor_down(size.height),
        KeyCode::Up | KeyCode::Char('k') => app.move_cursor_up(),
        KeyCode::Right | KeyCode::Char('l') => app.move_cursor_right(size.height),
        KeyCode::Left | KeyCode::Char('h') => app.move_cursor_left(),
        _ => {}
    }
    Ok(())
}
