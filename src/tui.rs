use crate::app::App;
use crate::event::EventHandler;
use crate::ui;
use crossterm::cursor::MoveTo;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;
use std::panic;

/// Representation of a terminal user interface.
#[derive(Debug)]
pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    pub events: EventHandler,
}

pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

impl<B: Backend> Tui<B> {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    /// Initializes the terminal interface.
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        // Define a custom panic hook to reset the terminal properties.
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    pub fn draw(&mut self, app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|frame| ui::render(app, frame))?;
        Ok(())
    }

    /// Resets the terminal interface.
    fn reset() -> Result<(), Box<dyn std::error::Error>> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(
            io::stderr(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            Clear(ClearType::All),
            MoveTo(0, 0)
        )?;
        Ok(())
    }

    /// Returns the size of the terminal interface.
    pub fn size(&self) -> TerminalSize {
        let size = self.terminal.size().unwrap();
        TerminalSize {
            width: size.width,
            height: size.height,
        }
    }

    /// Exits the terminal interface.
    pub fn exit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
