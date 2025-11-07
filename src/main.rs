mod app;
mod db;
mod ui;

use app::{App, InputMode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with database in ./tmp directory
    std::fs::create_dir_all("./tmp")?;
    let mut app = App::new("./tmp/todos.db")?;

    // Run app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
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
