use rust_ratatui_todo::app::App;
use rust_ratatui_todo::{event_handler, terminal};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    let mut term = terminal::setup()?;

    // Create app with database in ./tmp directory
    std::fs::create_dir_all("./tmp")?;
    let mut app = App::new("./tmp/todos.db")?;

    // Run app
    let res = event_handler::run(&mut term, &mut app);

    // Restore terminal
    terminal::restore(&mut term)?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}
