pub mod views;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

/// Color palette for the TUI — zero-cost constants.
pub mod theme {
    use ratatui::style::{Color, Modifier, Style};

    pub const ACCENT: Color = Color::Rgb(100, 200, 255);
    pub const SUCCESS: Color = Color::Rgb(100, 255, 150);
    pub const WARN: Color = Color::Rgb(255, 200, 80);
    pub const ERROR: Color = Color::Rgb(255, 100, 100);
    pub const DIM: Color = Color::DarkGray;
    pub const TEXT: Color = Color::White;
    pub const BG: Color = Color::Reset;

    pub const fn title() -> Style {
        Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)
    }

    pub const fn highlight() -> Style {
        Style::new()
            .fg(Color::Black)
            .bg(ACCENT)
            .add_modifier(Modifier::BOLD)
    }

    pub const fn success() -> Style {
        Style::new().fg(SUCCESS).add_modifier(Modifier::BOLD)
    }

    pub const fn warn() -> Style {
        Style::new().fg(WARN)
    }

    pub const fn error() -> Style {
        Style::new().fg(ERROR).add_modifier(Modifier::BOLD)
    }

    pub const fn dim() -> Style {
        Style::new().fg(DIM)
    }

    pub const fn normal() -> Style {
        Style::new().fg(TEXT)
    }
}

/// Keyboard input result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    Quit,
    Up,
    Down,
    Left,
    Right,
    Select,
    Back,
    Tab,
    Char(char),
    None,
}

/// Poll for a key event with a given timeout. Returns `InputAction`.
///
/// When `text_input` is true, printable characters are passed through
/// as `Char(c)` instead of being mapped to vim-style navigation or quit.
pub fn poll_input(timeout: Duration, text_input: bool) -> InputAction {
    if event::poll(timeout).unwrap_or(false) {
        if let Ok(Event::Key(key)) = event::read() {
            return map_key(key, text_input);
        }
    }
    InputAction::None
}

fn map_key(key: KeyEvent, text_input: bool) -> InputAction {
    // Ctrl+C / Ctrl+Q always quit
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') | KeyCode::Char('q') => InputAction::Quit,
            _ => InputAction::None,
        };
    }

    // In text input mode, only handle special keys; let all chars through.
    if text_input {
        return match key.code {
            KeyCode::Enter => InputAction::Select,
            KeyCode::Esc => InputAction::Back,
            KeyCode::Backspace => InputAction::Back,
            KeyCode::Char(c) => InputAction::Char(c),
            _ => InputAction::None,
        };
    }

    match key.code {
        KeyCode::Char('q') => InputAction::Quit,
        KeyCode::Up | KeyCode::Char('k') => InputAction::Up,
        KeyCode::Down | KeyCode::Char('j') => InputAction::Down,
        KeyCode::Left | KeyCode::Char('h') => InputAction::Left,
        KeyCode::Right | KeyCode::Char('l') => InputAction::Right,
        KeyCode::Enter => InputAction::Select,
        KeyCode::Esc | KeyCode::Backspace => InputAction::Back,
        KeyCode::Tab => InputAction::Tab,
        KeyCode::Char(c) => InputAction::Char(c),
        _ => InputAction::None,
    }
}

/// Initialize the terminal for TUI rendering.
pub fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to its original state.
pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
