use std::io::{self, Stdout, Write};

use anyhow::Result;
use ratatui::{backend::CrosstermBackend, Terminal};

pub type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

pub fn init_terminal() -> Result<AppTerminal> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
    set_mouse_capture(true)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

pub fn restore_terminal() -> Result<()> {
    set_mouse_capture(false)?;
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}

pub fn set_mouse_capture(enabled: bool) -> Result<()> {
    let mut stdout = io::stdout();
    if enabled {
        crossterm::execute!(stdout, crossterm::event::EnableMouseCapture)?;
        write!(stdout, "\x1b[?1003h")?;
    } else {
        crossterm::execute!(stdout, crossterm::event::DisableMouseCapture)?;
        write!(stdout, "\x1b[?1003l")?;
    }
    stdout.flush()?;
    Ok(())
}
