use crate::models::Todo;
use rusqlite::{Connection, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection and initialize the schema
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Database { conn };
        db.init()?;
        Ok(db)
    }

    /// Initialize the database schema
    fn init(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS todos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;
        Ok(())
    }

    /// Add a new todo item
    pub fn add_todo(&self, title: &str) -> Result<()> {
        if title.trim().is_empty() {
            return Err(rusqlite::Error::InvalidParameterName(
                "Title cannot be empty".to_string(),
            ));
        }
        self.conn.execute(
            "INSERT INTO todos (title, completed) VALUES (?1, 0)",
            [title],
        )?;
        Ok(())
    }

    /// Get all todo items ordered by ID
    pub fn get_todos(&self) -> Result<Vec<Todo>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, completed FROM todos ORDER BY id")?;
        let todos = stmt.query_map([], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                completed: row.get::<_, i64>(2)? != 0,
            })
        })?;

        todos.collect()
    }

    /// Toggle the completion status of a todo item
    pub fn toggle_todo(&self, id: i64) -> Result<()> {
        let rows_affected = self.conn.execute(
            "UPDATE todos SET completed = NOT completed WHERE id = ?1",
            [id],
        )?;
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    }

    /// Delete a todo item by ID
    pub fn delete_todo(&self, id: i64) -> Result<()> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM todos WHERE id = ?1", [id])?;
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    }

    /// Get a single todo by ID
    #[cfg(test)]
    pub fn get_todo_by_id(&self, id: i64) -> Result<Option<Todo>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, completed FROM todos WHERE id = ?1")?;
        let mut rows = stmt.query([id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                completed: row.get::<_, i64>(2)? != 0,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Database {
        Database::new(":memory:").expect("Failed to create in-memory database")
    }

    #[test]
    fn test_database_initialization() {
        let db = create_test_db();
        let todos = db.get_todos().expect("Failed to get todos");
        assert!(todos.is_empty());
    }

    #[test]
    fn test_add_todo() {
        let db = create_test_db();
        db.add_todo("Test todo").expect("Failed to add todo");

        let todos = db.get_todos().expect("Failed to get todos");
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "Test todo");
        assert!(!todos[0].completed);
    }

    #[test]
    fn test_add_empty_todo_fails() {
        let db = create_test_db();
        let result = db.add_todo("");
        assert!(result.is_err());

        let result = db.add_todo("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_multiple_todos() {
        let db = create_test_db();
        db.add_todo("First todo").expect("Failed to add todo");
        db.add_todo("Second todo").expect("Failed to add todo");
        db.add_todo("Third todo").expect("Failed to add todo");

        let todos = db.get_todos().expect("Failed to get todos");
        assert_eq!(todos.len(), 3);
        assert_eq!(todos[0].title, "First todo");
        assert_eq!(todos[1].title, "Second todo");
        assert_eq!(todos[2].title, "Third todo");
    }

    #[test]
    fn test_toggle_todo() {
        let db = create_test_db();
        db.add_todo("Test todo").expect("Failed to add todo");

        let todos = db.get_todos().expect("Failed to get todos");
        let todo_id = todos[0].id;
        assert!(!todos[0].completed);

        // Toggle to completed
        db.toggle_todo(todo_id).expect("Failed to toggle todo");
        let todos = db.get_todos().expect("Failed to get todos");
        assert!(todos[0].completed);

        // Toggle back to incomplete
        db.toggle_todo(todo_id).expect("Failed to toggle todo");
        let todos = db.get_todos().expect("Failed to get todos");
        assert!(!todos[0].completed);
    }

    #[test]
    fn test_toggle_nonexistent_todo() {
        let db = create_test_db();
        let result = db.toggle_todo(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_todo() {
        let db = create_test_db();
        db.add_todo("Test todo").expect("Failed to add todo");

        let todos = db.get_todos().expect("Failed to get todos");
        assert_eq!(todos.len(), 1);
        let todo_id = todos[0].id;

        db.delete_todo(todo_id).expect("Failed to delete todo");
        let todos = db.get_todos().expect("Failed to get todos");
        assert!(todos.is_empty());
    }

    #[test]
    fn test_delete_nonexistent_todo() {
        let db = create_test_db();
        let result = db.delete_todo(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_specific_todo() {
        let db = create_test_db();
        db.add_todo("First todo").expect("Failed to add todo");
        db.add_todo("Second todo").expect("Failed to add todo");
        db.add_todo("Third todo").expect("Failed to add todo");

        let todos = db.get_todos().expect("Failed to get todos");
        let middle_id = todos[1].id;

        db.delete_todo(middle_id)
            .expect("Failed to delete middle todo");
        let todos = db.get_todos().expect("Failed to get todos");
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].title, "First todo");
        assert_eq!(todos[1].title, "Third todo");
    }

    #[test]
    fn test_get_todo_by_id() {
        let db = create_test_db();
        db.add_todo("Test todo").expect("Failed to add todo");

        let todos = db.get_todos().expect("Failed to get todos");
        let todo_id = todos[0].id;

        let todo = db
            .get_todo_by_id(todo_id)
            .expect("Failed to get todo by ID");
        assert!(todo.is_some());
        assert_eq!(todo.unwrap().title, "Test todo");

        let nonexistent = db
            .get_todo_by_id(999)
            .expect("Failed to query nonexistent todo");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_todo_ordering() {
        let db = create_test_db();
        db.add_todo("First").expect("Failed to add todo");
        db.add_todo("Second").expect("Failed to add todo");
        db.add_todo("Third").expect("Failed to add todo");

        let todos = db.get_todos().expect("Failed to get todos");
        assert_eq!(todos.len(), 3);

        // Verify IDs are in ascending order
        for i in 1..todos.len() {
            assert!(todos[i].id > todos[i - 1].id);
        }
    }

    #[test]
    fn test_persistence() {
        use std::fs;
        let test_db_path = "/tmp/test_todo_db.sqlite";

        // Clean up any existing test database
        let _ = fs::remove_file(test_db_path);

        {
            let db = Database::new(test_db_path).expect("Failed to create database");
            db.add_todo("Persistent todo")
                .expect("Failed to add todo");
        }

        // Reopen the database
        {
            let db = Database::new(test_db_path).expect("Failed to reopen database");
            let todos = db.get_todos().expect("Failed to get todos");
            assert_eq!(todos.len(), 1);
            assert_eq!(todos[0].title, "Persistent todo");
        }

        // Clean up
        let _ = fs::remove_file(test_db_path);
    }
}
