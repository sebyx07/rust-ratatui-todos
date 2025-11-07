// Integration tests for database operations
// Tests the public API of the database module

use rust_ratatui_todo::db::Database;
use std::env;
use std::fs;

/// Helper to create a unique test database path
fn test_db_path(name: &str) -> String {
    env::temp_dir()
        .join(format!("test_db_integration_{}.db", name))
        .to_str()
        .expect("Invalid path")
        .to_string()
}

/// Helper to clean up test database
fn cleanup_db(path: &str) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(format!("{}-shm", path));
    let _ = fs::remove_file(format!("{}-wal", path));
}

#[test]
fn test_database_creation_and_persistence() {
    let db_path = test_db_path("creation");
    cleanup_db(&db_path);

    // Create database and add data
    {
        let db = Database::new(&db_path).expect("Failed to create database");
        db.add_todo("Persistent todo").expect("Failed to add todo");
        db.add_todo("Another todo").expect("Failed to add todo");

        let todos = db.get_todos().expect("Failed to get todos");
        assert_eq!(todos.len(), 2);
    }

    // Reopen database and verify persistence
    {
        let db = Database::new(&db_path).expect("Failed to reopen database");
        let todos = db.get_todos().expect("Failed to get todos");
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].title, "Persistent todo");
        assert_eq!(todos[1].title, "Another todo");
    }

    cleanup_db(&db_path);
}

#[test]
fn test_concurrent_database_operations() {
    let db_path = test_db_path("concurrent");
    cleanup_db(&db_path);

    let db = Database::new(&db_path).expect("Failed to create database");

    // Add multiple todos
    for i in 1..=10 {
        db.add_todo(&format!("Todo {}", i))
            .expect("Failed to add todo");
    }

    let todos = db.get_todos().expect("Failed to get todos");
    assert_eq!(todos.len(), 10);

    // Toggle multiple todos
    for (idx, todo) in todos.iter().enumerate() {
        if idx % 2 == 0 {
            db.toggle_todo(todo.id).expect("Failed to toggle todo");
        }
    }

    let todos = db.get_todos().expect("Failed to get todos");
    let completed_count = todos.iter().filter(|t| t.completed).count();
    assert_eq!(completed_count, 5);

    cleanup_db(&db_path);
}

#[test]
fn test_database_error_handling() {
    let db_path = test_db_path("errors");
    cleanup_db(&db_path);

    let db = Database::new(&db_path).expect("Failed to create database");

    // Test empty title validation
    let result = db.add_todo("");
    assert!(result.is_err(), "Should fail to add empty todo");

    let result = db.add_todo("   ");
    assert!(result.is_err(), "Should fail to add whitespace-only todo");

    // Test operations on nonexistent todos
    let result = db.toggle_todo(999);
    assert!(result.is_err(), "Should fail to toggle nonexistent todo");

    let result = db.delete_todo(999);
    assert!(result.is_err(), "Should fail to delete nonexistent todo");

    cleanup_db(&db_path);
}

#[test]
fn test_database_transaction_like_behavior() {
    let db_path = test_db_path("transactions");
    cleanup_db(&db_path);

    let db = Database::new(&db_path).expect("Failed to create database");

    // Add several todos
    db.add_todo("Task 1").unwrap();
    db.add_todo("Task 2").unwrap();
    db.add_todo("Task 3").unwrap();

    let todos = db.get_todos().unwrap();
    let task2_id = todos[1].id;

    // Delete middle item
    db.delete_todo(task2_id).unwrap();

    // Verify the remaining tasks
    let todos = db.get_todos().unwrap();
    assert_eq!(todos.len(), 2);
    assert_eq!(todos[0].title, "Task 1");
    assert_eq!(todos[1].title, "Task 3");

    // Add new task and verify it gets a new ID
    db.add_todo("Task 4").unwrap();
    let todos = db.get_todos().unwrap();
    assert_eq!(todos.len(), 3);
    assert_eq!(todos[2].title, "Task 4");

    cleanup_db(&db_path);
}

#[test]
fn test_database_with_special_characters() {
    let db_path = test_db_path("special_chars");
    cleanup_db(&db_path);

    let db = Database::new(&db_path).expect("Failed to create database");

    let special_titles = vec![
        "Todo with 'single quotes'",
        "Todo with \"double quotes\"",
        "Todo with emoji 🚀",
        "Todo with newline\ncharacter",
        "Todo with tab\tcharacter",
        "Todo with unicode: 你好世界",
        "Todo with SQL-like syntax: SELECT * FROM todos; DROP TABLE todos;",
    ];

    for title in &special_titles {
        db.add_todo(title)
            .expect("Failed to add todo with special chars");
    }

    let todos = db.get_todos().expect("Failed to get todos");
    assert_eq!(todos.len(), special_titles.len());

    for (idx, todo) in todos.iter().enumerate() {
        assert_eq!(todo.title, special_titles[idx]);
    }

    cleanup_db(&db_path);
}

#[test]
fn test_database_large_dataset() {
    let db_path = test_db_path("large_dataset");
    cleanup_db(&db_path);

    let db = Database::new(&db_path).expect("Failed to create database");

    // Add 100 todos
    for i in 1..=100 {
        db.add_todo(&format!("Todo number {}", i))
            .expect("Failed to add todo");
    }

    let todos = db.get_todos().expect("Failed to get todos");
    assert_eq!(todos.len(), 100);

    // Verify ordering
    for (idx, todo) in todos.iter().enumerate() {
        assert_eq!(todo.title, format!("Todo number {}", idx + 1));
    }

    // Toggle every 10th todo
    for i in (0..100).step_by(10) {
        db.toggle_todo(todos[i].id).expect("Failed to toggle");
    }

    let todos = db.get_todos().expect("Failed to get todos");
    let completed = todos.iter().filter(|t| t.completed).count();
    assert_eq!(completed, 10);

    cleanup_db(&db_path);
}
