use crate::db::Database;
use crate::models::{InputMode, Todo};
use ratatui::widgets::ListState;
use std::error::Error;

pub struct App {
    pub db: Database,
    pub todos: Vec<Todo>,
    pub list_state: ListState,
    pub input_mode: InputMode,
    pub input: String,
}

impl App {
    /// Create a new App instance with the given database path
    pub fn new(db_path: &str) -> Result<Self, Box<dyn Error>> {
        let db = Database::new(db_path)?;
        let todos = db.get_todos()?;
        let mut list_state = ListState::default();
        if !todos.is_empty() {
            list_state.select(Some(0));
        }

        Ok(App {
            db,
            todos,
            list_state,
            input_mode: InputMode::Normal,
            input: String::new(),
        })
    }

    /// Refresh todos from the database
    pub fn refresh_todos(&mut self) -> Result<(), Box<dyn Error>> {
        self.todos = self.db.get_todos()?;

        // Adjust selection if needed
        if self.todos.is_empty() {
            self.list_state.select(None);
        } else if let Some(selected) = self.list_state.selected() {
            if selected >= self.todos.len() {
                self.list_state.select(Some(self.todos.len() - 1));
            }
        } else if !self.todos.is_empty() {
            self.list_state.select(Some(0));
        }

        Ok(())
    }

    /// Move selection to the next item
    pub fn next(&mut self) {
        if self.todos.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.todos.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Move selection to the previous item
    pub fn previous(&mut self) {
        if self.todos.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.todos.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Toggle the completion status of the selected todo
    pub fn toggle_selected(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(i) = self.list_state.selected()
            && let Some(todo) = self.todos.get(i)
        {
            self.db.toggle_todo(todo.id)?;
            self.refresh_todos()?;
        }
        Ok(())
    }

    /// Delete the selected todo
    pub fn delete_selected(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(i) = self.list_state.selected()
            && let Some(todo) = self.todos.get(i)
        {
            self.db.delete_todo(todo.id)?;
            self.refresh_todos()?;
        }
        Ok(())
    }

    /// Add a new todo from the current input
    pub fn add_todo(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.input.trim().is_empty() {
            self.db.add_todo(&self.input)?;
            self.input.clear();
            self.refresh_todos()?;
        }
        Ok(())
    }

    /// Enter editing mode
    pub fn enter_edit_mode(&mut self) {
        self.input_mode = InputMode::Editing;
    }

    /// Exit editing mode and clear input
    pub fn exit_edit_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input.clear();
    }

    /// Add a character to the input
    pub fn input_char(&mut self, c: char) {
        self.input.push(c);
    }

    /// Remove the last character from the input
    pub fn input_backspace(&mut self) {
        self.input.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_app() -> App {
        App::new(":memory:").expect("Failed to create test app")
    }

    #[test]
    fn test_app_initialization() {
        let app = create_test_app();
        assert!(app.todos.is_empty());
        assert!(app.list_state.selected().is_none());
        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.input.is_empty());
    }

    #[test]
    fn test_add_todo_through_app() {
        let mut app = create_test_app();
        app.input = "Test todo".to_string();
        app.add_todo().expect("Failed to add todo");

        assert!(app.input.is_empty());
        assert_eq!(app.todos.len(), 1);
        assert_eq!(app.todos[0].title, "Test todo");
    }

    #[test]
    fn test_navigation() {
        let mut app = create_test_app();
        app.db.add_todo("First").unwrap();
        app.db.add_todo("Second").unwrap();
        app.db.add_todo("Third").unwrap();
        app.refresh_todos().unwrap();

        assert_eq!(app.list_state.selected(), Some(0));

        app.next();
        assert_eq!(app.list_state.selected(), Some(1));

        app.next();
        assert_eq!(app.list_state.selected(), Some(2));

        // Wrap around
        app.next();
        assert_eq!(app.list_state.selected(), Some(0));

        app.previous();
        assert_eq!(app.list_state.selected(), Some(2));
    }

    #[test]
    fn test_toggle_selected() {
        let mut app = create_test_app();
        app.db.add_todo("Test todo").unwrap();
        app.refresh_todos().unwrap();

        assert!(!app.todos[0].completed);

        app.toggle_selected().expect("Failed to toggle todo");
        assert!(app.todos[0].completed);

        app.toggle_selected().expect("Failed to toggle todo");
        assert!(!app.todos[0].completed);
    }

    #[test]
    fn test_delete_selected() {
        let mut app = create_test_app();
        app.db.add_todo("Test todo").unwrap();
        app.refresh_todos().unwrap();

        assert_eq!(app.todos.len(), 1);

        app.delete_selected().expect("Failed to delete todo");
        assert!(app.todos.is_empty());
        assert!(app.list_state.selected().is_none());
    }

    #[test]
    fn test_input_mode_transitions() {
        let mut app = create_test_app();
        assert_eq!(app.input_mode, InputMode::Normal);

        app.enter_edit_mode();
        assert_eq!(app.input_mode, InputMode::Editing);

        app.exit_edit_mode();
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_input_operations() {
        let mut app = create_test_app();

        app.input_char('H');
        app.input_char('e');
        app.input_char('l');
        app.input_char('l');
        app.input_char('o');
        assert_eq!(app.input, "Hello");

        app.input_backspace();
        assert_eq!(app.input, "Hell");

        app.input_backspace();
        app.input_backspace();
        app.input_backspace();
        app.input_backspace();
        assert!(app.input.is_empty());

        // Backspace on empty string should not panic
        app.input_backspace();
        assert!(app.input.is_empty());
    }
}
