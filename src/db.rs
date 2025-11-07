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

    /// Get paginated todo items ordered by ID
    pub fn get_todos_paginated(&self, page: u32, page_size: u32) -> Result<Vec<Todo>> {
        let offset = page * page_size;
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, completed FROM todos ORDER BY id LIMIT ?1 OFFSET ?2")?;
        let todos = stmt.query_map([page_size, offset], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                completed: row.get::<_, i64>(2)? != 0,
            })
        })?;

        todos.collect()
    }

    /// Get total count of todos
    pub fn count_todos(&self) -> Result<u32> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM todos", [], |row| row.get(0))?;
        Ok(count as u32)
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
        let rows_affected = self.conn.execute("DELETE FROM todos WHERE id = ?1", [id])?;
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    }

    /// Get a single todo by ID
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

    /// Update the title of a todo item
    pub fn update_todo_title(&self, id: i64, title: &str) -> Result<()> {
        if title.trim().is_empty() {
            return Err(rusqlite::Error::InvalidParameterName(
                "Title cannot be empty".to_string(),
            ));
        }
        let rows_affected = self.conn.execute(
            "UPDATE todos SET title = ?1 WHERE id = ?2",
            [title, &id.to_string()],
        )?;
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    }

    /// Clear all todos from the database
    pub fn clear_all(&self) -> Result<u32> {
        let rows_affected = self.conn.execute("DELETE FROM todos", [])?;
        Ok(rows_affected as u32)
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
        use std::env;
        let test_db_path = env::temp_dir().join("test_todo_db.sqlite");
        let test_db_path_str = test_db_path.to_str().expect("Invalid path");

        // Clean up any existing test database
        let _ = fs::remove_file(&test_db_path);

        {
            let db = Database::new(test_db_path_str).expect("Failed to create database");
            db.add_todo("Persistent todo").expect("Failed to add todo");
        }

        // Reopen the database
        {
            let db = Database::new(test_db_path_str).expect("Failed to reopen database");
            let todos = db.get_todos().expect("Failed to get todos");
            assert_eq!(todos.len(), 1);
            assert_eq!(todos[0].title, "Persistent todo");
        }

        // Clean up
        let _ = fs::remove_file(&test_db_path);
    }

    #[test]
    fn test_update_todo_title() {
        let db = create_test_db();
        db.add_todo("Original title").expect("Failed to add todo");

        let todos = db.get_todos().expect("Failed to get todos");
        let todo_id = todos[0].id;
        assert_eq!(todos[0].title, "Original title");

        // Update the title
        db.update_todo_title(todo_id, "Updated title")
            .expect("Failed to update title");

        let todos = db.get_todos().expect("Failed to get todos");
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, todo_id); // ID should remain the same
        assert_eq!(todos[0].title, "Updated title");
    }

    #[test]
    fn test_update_todo_title_empty_fails() {
        let db = create_test_db();
        db.add_todo("Original title").expect("Failed to add todo");

        let todos = db.get_todos().expect("Failed to get todos");
        let todo_id = todos[0].id;

        // Try to update with empty title
        let result = db.update_todo_title(todo_id, "");
        assert!(result.is_err());

        // Try to update with whitespace-only title
        let result = db.update_todo_title(todo_id, "   ");
        assert!(result.is_err());

        // Verify original title is unchanged
        let todos = db.get_todos().expect("Failed to get todos");
        assert_eq!(todos[0].title, "Original title");
    }

    #[test]
    fn test_update_nonexistent_todo_title() {
        let db = create_test_db();
        let result = db.update_todo_title(999, "New title");
        assert!(result.is_err());
    }

    #[test]
    fn test_update_todo_title_preserves_completion_status() {
        let db = create_test_db();
        db.add_todo("Original title").expect("Failed to add todo");

        let todos = db.get_todos().expect("Failed to get todos");
        let todo_id = todos[0].id;

        // Toggle to completed
        db.toggle_todo(todo_id).expect("Failed to toggle todo");

        // Update title
        db.update_todo_title(todo_id, "New title")
            .expect("Failed to update title");

        // Verify both title and completion status
        let todos = db.get_todos().expect("Failed to get todos");
        assert_eq!(todos[0].title, "New title");
        assert!(todos[0].completed);
    }

    #[test]
    fn test_get_todos_paginated() {
        let db = create_test_db();

        // Add 25 todos
        for i in 1..=25 {
            db.add_todo(&format!("Todo {}", i))
                .expect("Failed to add todo");
        }

        // Get first page (10 items)
        let page1 = db.get_todos_paginated(0, 10).expect("Failed to get page 1");
        assert_eq!(page1.len(), 10);
        assert_eq!(page1[0].title, "Todo 1");
        assert_eq!(page1[9].title, "Todo 10");

        // Get second page (10 items)
        let page2 = db.get_todos_paginated(1, 10).expect("Failed to get page 2");
        assert_eq!(page2.len(), 10);
        assert_eq!(page2[0].title, "Todo 11");
        assert_eq!(page2[9].title, "Todo 20");

        // Get third page (5 items)
        let page3 = db.get_todos_paginated(2, 10).expect("Failed to get page 3");
        assert_eq!(page3.len(), 5);
        assert_eq!(page3[0].title, "Todo 21");
        assert_eq!(page3[4].title, "Todo 25");

        // Get page beyond available data
        let page4 = db.get_todos_paginated(3, 10).expect("Failed to get page 4");
        assert_eq!(page4.len(), 0);
    }

    #[test]
    fn test_count_todos() {
        let db = create_test_db();

        // Initially empty
        let count = db.count_todos().expect("Failed to count todos");
        assert_eq!(count, 0);

        // Add 10 todos
        for i in 1..=10 {
            db.add_todo(&format!("Todo {}", i))
                .expect("Failed to add todo");
        }

        let count = db.count_todos().expect("Failed to count todos");
        assert_eq!(count, 10);

        // Delete one
        let todos = db.get_todos().expect("Failed to get todos");
        db.delete_todo(todos[0].id).expect("Failed to delete todo");

        let count = db.count_todos().expect("Failed to count todos");
        assert_eq!(count, 9);
    }

    #[test]
    fn test_pagination_with_different_page_sizes() {
        let db = create_test_db();

        // Add 100 todos
        for i in 1..=100 {
            db.add_todo(&format!("Todo {}", i))
                .expect("Failed to add todo");
        }

        // Test page size of 20
        let page1 = db.get_todos_paginated(0, 20).expect("Failed to get page");
        assert_eq!(page1.len(), 20);

        // Test page size of 50
        let page1 = db.get_todos_paginated(0, 50).expect("Failed to get page");
        assert_eq!(page1.len(), 50);

        // Test page size of 100
        let page1 = db.get_todos_paginated(0, 100).expect("Failed to get page");
        assert_eq!(page1.len(), 100);

        // Test page size larger than available
        let page1 = db.get_todos_paginated(0, 200).expect("Failed to get page");
        assert_eq!(page1.len(), 100);
    }
}
