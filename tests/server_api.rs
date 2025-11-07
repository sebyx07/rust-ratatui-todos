use rust_ratatui_todo::{db::Database, models::Todo};
use serde_json::json;
use std::sync::{Arc, Mutex};

/// Helper to create a test database
fn create_test_db() -> Database {
    Database::new(":memory:").expect("Failed to create in-memory database")
}

#[test]
fn test_database_operations_for_server() {
    let db = create_test_db();

    // Test create
    db.add_todo("Test todo").expect("Failed to add todo");
    let todos = db.get_todos().expect("Failed to get todos");
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].title, "Test todo");
    assert!(!todos[0].completed);

    let todo_id = todos[0].id;

    // Test get by ID
    let todo = db
        .get_todo_by_id(todo_id)
        .expect("Failed to get todo")
        .expect("Todo not found");
    assert_eq!(todo.title, "Test todo");

    // Test update title
    db.update_todo_title(todo_id, "Updated todo")
        .expect("Failed to update title");
    let todo = db
        .get_todo_by_id(todo_id)
        .expect("Failed to get todo")
        .expect("Todo not found");
    assert_eq!(todo.title, "Updated todo");

    // Test toggle
    db.toggle_todo(todo_id).expect("Failed to toggle todo");
    let todo = db
        .get_todo_by_id(todo_id)
        .expect("Failed to get todo")
        .expect("Todo not found");
    assert!(todo.completed);

    // Test delete
    db.delete_todo(todo_id).expect("Failed to delete todo");
    let todo = db.get_todo_by_id(todo_id).expect("Failed to query");
    assert!(todo.is_none());
}

#[test]
fn test_concurrent_database_access() {
    let db = Arc::new(Mutex::new(create_test_db()));

    // Simulate multiple concurrent requests
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let db_clone = Arc::clone(&db);
            std::thread::spawn(move || {
                let db = db_clone.lock().unwrap();
                db.add_todo(&format!("Todo {}", i))
                    .expect("Failed to add todo");
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let db = db.lock().unwrap();
    let todos = db.get_todos().expect("Failed to get todos");
    assert_eq!(todos.len(), 10);
}

#[test]
fn test_json_serialization() {
    let todo = Todo {
        id: 1,
        title: "Test todo".to_string(),
        completed: false,
    };

    // Test serialization
    let json = serde_json::to_string(&todo).expect("Failed to serialize");
    assert!(json.contains("\"id\":1"));
    assert!(json.contains("\"title\":\"Test todo\""));
    assert!(json.contains("\"completed\":false"));

    // Test deserialization
    let parsed: Todo = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(parsed.id, 1);
    assert_eq!(parsed.title, "Test todo");
    assert!(!parsed.completed);
}

#[test]
fn test_create_request_validation() {
    // Valid request
    let valid = json!({"title": "Valid todo"});
    assert!(valid.get("title").is_some());

    // Empty title should be rejected by database layer
    let db = create_test_db();
    assert!(db.add_todo("").is_err());
    assert!(db.add_todo("   ").is_err());
}

#[test]
fn test_update_request_validation() {
    let db = create_test_db();
    db.add_todo("Original").expect("Failed to add todo");

    let todos = db.get_todos().expect("Failed to get todos");
    let todo_id = todos[0].id;

    // Valid update
    db.update_todo_title(todo_id, "Updated")
        .expect("Failed to update");

    // Invalid update (empty title)
    assert!(db.update_todo_title(todo_id, "").is_err());
    assert!(db.update_todo_title(todo_id, "   ").is_err());
}

#[test]
fn test_error_cases() {
    let db = create_test_db();

    // Get non-existent todo
    let result = db.get_todo_by_id(999).expect("Failed to query");
    assert!(result.is_none());

    // Update non-existent todo
    assert!(db.update_todo_title(999, "Title").is_err());

    // Toggle non-existent todo
    assert!(db.toggle_todo(999).is_err());

    // Delete non-existent todo
    assert!(db.delete_todo(999).is_err());
}

#[test]
fn test_multiple_operations_workflow() {
    let db = create_test_db();

    // Create multiple todos
    db.add_todo("Todo 1").expect("Failed to add");
    db.add_todo("Todo 2").expect("Failed to add");
    db.add_todo("Todo 3").expect("Failed to add");

    let todos = db.get_todos().expect("Failed to get todos");
    assert_eq!(todos.len(), 3);

    // Update the middle one
    let middle_id = todos[1].id;
    db.update_todo_title(middle_id, "Updated Todo 2")
        .expect("Failed to update");

    // Toggle first and last
    db.toggle_todo(todos[0].id).expect("Failed to toggle");
    db.toggle_todo(todos[2].id).expect("Failed to toggle");

    // Verify state
    let todos = db.get_todos().expect("Failed to get todos");
    assert_eq!(todos.len(), 3);
    assert!(todos[0].completed);
    assert_eq!(todos[1].title, "Updated Todo 2");
    assert!(!todos[1].completed);
    assert!(todos[2].completed);

    // Delete the middle one
    db.delete_todo(middle_id).expect("Failed to delete");
    let todos = db.get_todos().expect("Failed to get todos");
    assert_eq!(todos.len(), 2);
    assert_eq!(todos[0].title, "Todo 1");
    assert_eq!(todos[1].title, "Todo 3");
}

#[test]
fn test_thread_safety_with_arc_mutex() {
    let db = Arc::new(Mutex::new(create_test_db()));

    // Add initial todos
    {
        let db = db.lock().unwrap();
        db.add_todo("Initial 1").expect("Failed to add");
        db.add_todo("Initial 2").expect("Failed to add");
    }

    // Concurrent reads
    let read_handles: Vec<_> = (0..5)
        .map(|_| {
            let db_clone = Arc::clone(&db);
            std::thread::spawn(move || {
                let db = db_clone.lock().unwrap();
                let todos = db.get_todos().expect("Failed to get todos");
                assert!(todos.len() >= 2);
            })
        })
        .collect();

    for handle in read_handles {
        handle.join().expect("Thread panicked");
    }

    // Concurrent writes and reads
    let mixed_handles: Vec<_> = (0..10)
        .map(|i| {
            let db_clone = Arc::clone(&db);
            std::thread::spawn(move || {
                let db = db_clone.lock().unwrap();
                if i % 2 == 0 {
                    db.add_todo(&format!("Thread {}", i))
                        .expect("Failed to add");
                } else {
                    let _ = db.get_todos();
                }
            })
        })
        .collect();

    for handle in mixed_handles {
        handle.join().expect("Thread panicked");
    }

    // Verify final state
    let db = db.lock().unwrap();
    let todos = db.get_todos().expect("Failed to get todos");
    assert!(todos.len() >= 7); // 2 initial + at least 5 from threads
}

#[test]
fn test_update_preserves_other_fields() {
    let db = create_test_db();
    db.add_todo("Original title").expect("Failed to add");

    let todos = db.get_todos().expect("Failed to get todos");
    let todo_id = todos[0].id;

    // Mark as completed
    db.toggle_todo(todo_id).expect("Failed to toggle");

    // Update title
    db.update_todo_title(todo_id, "New title")
        .expect("Failed to update");

    // Verify both ID and completion are preserved
    let todo = db
        .get_todo_by_id(todo_id)
        .expect("Failed to get")
        .expect("Not found");
    assert_eq!(todo.id, todo_id);
    assert_eq!(todo.title, "New title");
    assert!(todo.completed);
}

#[test]
fn test_toggle_preserves_title() {
    let db = create_test_db();
    db.add_todo("Test title").expect("Failed to add");

    let todos = db.get_todos().expect("Failed to get todos");
    let todo_id = todos[0].id;

    // Toggle multiple times
    db.toggle_todo(todo_id).expect("Failed to toggle");
    db.toggle_todo(todo_id).expect("Failed to toggle");
    db.toggle_todo(todo_id).expect("Failed to toggle");

    // Verify title is preserved
    let todo = db
        .get_todo_by_id(todo_id)
        .expect("Failed to get")
        .expect("Not found");
    assert_eq!(todo.title, "Test title");
    assert!(todo.completed); // Toggled odd number of times
}
