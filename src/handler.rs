use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    app::App,
    search::{SearchKind, SearchMode},
    tui::TerminalSize,
};

/// Dispatch key events based on the current application mode.
pub fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    size: TerminalSize,
) -> Result<(), Box<dyn std::error::Error>> {
    match app.search.mode {
        SearchMode::Input(_) => handle_search_input(key_event, app, size),
        SearchMode::Normal => handle_normal_mode(key_event, app, size),
    }
    Ok(())
}

fn handle_normal_mode(key_event: KeyEvent, app: &mut App, size: TerminalSize) {
    match key_event.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Down | KeyCode::Char('j') => app.move_cursor_down(size.height),
        KeyCode::Up | KeyCode::Char('k') => app.move_cursor_up(),
        KeyCode::Right | KeyCode::Char('l') => app.move_cursor_right(size.height),
        KeyCode::Left | KeyCode::Char('h') => app.move_cursor_left(),
        KeyCode::Char('/') => app.start_search(SearchKind::Hex),
        KeyCode::Char('?') => app.start_search(SearchKind::Ascii),
        KeyCode::Char('n') => app.next_match(size.height),
        KeyCode::Char('N') | KeyCode::Char('p') => app.prev_match(size.height),
        KeyCode::Esc => app.search.cancel(),
        _ => {}
    }
}

fn handle_search_input(key_event: KeyEvent, app: &mut App, size: TerminalSize) {
    match key_event.code {
        KeyCode::Enter => app.submit_search(size.height),
        KeyCode::Esc => app.search.cancel(),
        KeyCode::Backspace => {
            app.search.query.pop();
        }
        KeyCode::Tab | KeyCode::BackTab => app.search.toggle_kind(),
        KeyCode::Char(c) => {
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                return;
            }
            app.search.query.push(c);
        }
        _ => {}
    }
}
