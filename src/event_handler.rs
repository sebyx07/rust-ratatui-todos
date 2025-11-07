use crate::app::App;
use crate::models::InputMode;
use crate::ui;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::Terminal;
use std::error::Error;

/// Run the main event loop for the application
pub fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('i') | KeyCode::Char('a') => {
                        app.enter_edit_mode();
                    }
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),
                    KeyCode::Char(' ') => app.toggle_selected()?,
                    KeyCode::Char('d') | KeyCode::Delete => app.delete_selected()?,
                    KeyCode::Char('n') | KeyCode::PageDown => app.next_page()?,
                    KeyCode::Char('p') | KeyCode::PageUp => app.previous_page()?,
                    KeyCode::Char('C') => {
                        // Capital C to clear all (requires shift, prevents accidental deletion)
                        app.clear_all()?;
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        app.add_todo()?;
                    }
                    KeyCode::Char(c) => {
                        app.input_char(c);
                    }
                    KeyCode::Backspace => {
                        app.input_backspace();
                    }
                    KeyCode::Esc => {
                        app.exit_edit_mode();
                    }
                    _ => {}
                },
            }
        }
    }
}
