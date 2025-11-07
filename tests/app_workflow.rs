// Integration tests for application workflows
// Tests the interaction between app and database modules

use rust_ratatui_todo::app::App;
use rust_ratatui_todo::models::InputMode;
use std::fs;

/// Helper to create a unique test database path
fn test_db_path(name: &str) -> String {
    format!("/tmp/test_app_workflow_{}.db", name)
}

/// Helper to clean up test database
fn cleanup_db(path: &str) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(format!("{}-shm", path));
    let _ = fs::remove_file(format!("{}-wal", path));
}

#[test]
fn test_app_initialization_with_persistence() {
    let db_path = test_db_path("init_persist");
    cleanup_db(&db_path);

    // Create app and add todos
    {
        let mut app = App::new(&db_path).expect("Failed to create app");
        assert!(app.todos.is_empty());

        app.input = "First todo".to_string();
        app.add_todo().expect("Failed to add todo");

        app.input = "Second todo".to_string();
        app.add_todo().expect("Failed to add todo");

        assert_eq!(app.todos.len(), 2);
    }

    // Reopen app and verify data persisted
    {
        let app = App::new(&db_path).expect("Failed to reopen app");
        assert_eq!(app.todos.len(), 2);
        assert_eq!(app.todos[0].title, "First todo");
        assert_eq!(app.todos[1].title, "Second todo");
        assert!(app.list_state.selected().is_some());
    }

    cleanup_db(&db_path);
}

#[test]
fn test_complete_user_workflow() {
    let db_path = test_db_path("complete_workflow");
    cleanup_db(&db_path);

    let mut app = App::new(&db_path).expect("Failed to create app");

    // Start in normal mode
    assert_eq!(app.input_mode, InputMode::Normal);
    assert!(app.todos.is_empty());

    // Enter edit mode and add first todo
    app.enter_edit_mode();
    assert_eq!(app.input_mode, InputMode::Editing);

    "Buy groceries".chars().for_each(|c| app.input_char(c));
    assert_eq!(app.input, "Buy groceries");

    app.add_todo().expect("Failed to add todo");
    assert_eq!(app.todos.len(), 1);
    assert!(app.input.is_empty());

    // Add second todo
    "Write documentation"
        .chars()
        .for_each(|c| app.input_char(c));
    app.add_todo().expect("Failed to add todo");
    assert_eq!(app.todos.len(), 2);

    // Exit edit mode
    app.exit_edit_mode();
    assert_eq!(app.input_mode, InputMode::Normal);

    // Navigate and toggle first todo
    assert_eq!(app.list_state.selected(), Some(0));
    assert!(!app.todos[0].completed);

    app.toggle_selected().expect("Failed to toggle todo");
    assert!(app.todos[0].completed);

    // Navigate to second todo
    app.next();
    assert_eq!(app.list_state.selected(), Some(1));

    // Delete second todo
    app.delete_selected().expect("Failed to delete todo");
    assert_eq!(app.todos.len(), 1);
    assert_eq!(app.todos[0].title, "Buy groceries");
    assert!(app.todos[0].completed);

    cleanup_db(&db_path);
}

#[test]
fn test_navigation_with_multiple_todos() {
    let db_path = test_db_path("navigation");
    cleanup_db(&db_path);

    let mut app = App::new(&db_path).expect("Failed to create app");

    // Add 5 todos
    for i in 1..=5 {
        app.input = format!("Todo {}", i);
        app.add_todo().expect("Failed to add todo");
    }

    assert_eq!(app.todos.len(), 5);
    assert_eq!(app.list_state.selected(), Some(0));

    // Navigate forward
    app.next();
    assert_eq!(app.list_state.selected(), Some(1));

    app.next();
    app.next();
    assert_eq!(app.list_state.selected(), Some(3));

    // Wrap around forward
    app.next();
    assert_eq!(app.list_state.selected(), Some(4));
    app.next();
    assert_eq!(app.list_state.selected(), Some(0));

    // Navigate backward
    app.previous();
    assert_eq!(app.list_state.selected(), Some(4));

    app.previous();
    app.previous();
    assert_eq!(app.list_state.selected(), Some(2));

    cleanup_db(&db_path);
}

#[test]
fn test_input_mode_with_cancel() {
    let db_path = test_db_path("input_cancel");
    cleanup_db(&db_path);

    let mut app = App::new(&db_path).expect("Failed to create app");

    // Enter edit mode and type
    app.enter_edit_mode();
    "This will be cancelled"
        .chars()
        .for_each(|c| app.input_char(c));
    assert_eq!(app.input, "This will be cancelled");

    // Cancel with exit_edit_mode
    app.exit_edit_mode();
    assert!(app.input.is_empty());
    assert_eq!(app.input_mode, InputMode::Normal);
    assert!(app.todos.is_empty());

    // Try again but save this time
    app.enter_edit_mode();
    "This will be saved".chars().for_each(|c| app.input_char(c));
    app.add_todo().expect("Failed to add todo");

    assert_eq!(app.todos.len(), 1);
    assert_eq!(app.todos[0].title, "This will be saved");
    assert!(app.input.is_empty());

    cleanup_db(&db_path);
}

#[test]
fn test_input_backspace() {
    let db_path = test_db_path("backspace");
    cleanup_db(&db_path);

    let mut app = App::new(&db_path).expect("Failed to create app");

    app.enter_edit_mode();

    // Type and use backspace
    "Hello World".chars().for_each(|c| app.input_char(c));
    assert_eq!(app.input, "Hello World");

    // Remove "World"
    for _ in 0..5 {
        app.input_backspace();
    }
    assert_eq!(app.input, "Hello ");

    // Remove space
    app.input_backspace();
    assert_eq!(app.input, "Hello");

    // Add different text
    " Rust".chars().for_each(|c| app.input_char(c));
    assert_eq!(app.input, "Hello Rust");

    app.add_todo().expect("Failed to add todo");
    assert_eq!(app.todos[0].title, "Hello Rust");

    cleanup_db(&db_path);
}

#[test]
fn test_delete_with_selection_adjustment() {
    let db_path = test_db_path("delete_adjust");
    cleanup_db(&db_path);

    let mut app = App::new(&db_path).expect("Failed to create app");

    // Add 3 todos
    for i in 1..=3 {
        app.input = format!("Todo {}", i);
        app.add_todo().expect("Failed to add todo");
    }

    // Navigate to last item
    app.next();
    app.next();
    assert_eq!(app.list_state.selected(), Some(2));

    // Delete last item - selection should move to new last item
    app.delete_selected().expect("Failed to delete todo");
    assert_eq!(app.todos.len(), 2);
    assert_eq!(app.list_state.selected(), Some(1));

    // Delete all remaining
    app.delete_selected().expect("Failed to delete todo");
    assert_eq!(app.todos.len(), 1);
    assert_eq!(app.list_state.selected(), Some(0));

    app.delete_selected().expect("Failed to delete todo");
    assert!(app.todos.is_empty());
    assert!(app.list_state.selected().is_none());

    cleanup_db(&db_path);
}

#[test]
fn test_toggle_multiple_todos() {
    let db_path = test_db_path("toggle_multiple");
    cleanup_db(&db_path);

    let mut app = App::new(&db_path).expect("Failed to create app");

    // Add 5 todos
    for i in 1..=5 {
        app.input = format!("Todo {}", i);
        app.add_todo().expect("Failed to add todo");
    }

    // Toggle every other todo
    for i in 0..5 {
        if i % 2 == 0 {
            app.toggle_selected().expect("Failed to toggle");
        }
        if i < 4 {
            app.next();
        }
    }

    // Verify completion status
    assert!(app.todos[0].completed);
    assert!(!app.todos[1].completed);
    assert!(app.todos[2].completed);
    assert!(!app.todos[3].completed);
    assert!(app.todos[4].completed);

    // Toggle them all back
    for i in 0..5 {
        app.list_state.select(Some(i));
        if app.todos[i].completed {
            app.toggle_selected().expect("Failed to toggle");
        }
    }

    // All should be incomplete
    for todo in &app.todos {
        assert!(!todo.completed);
    }

    cleanup_db(&db_path);
}

#[test]
fn test_empty_todo_prevention() {
    let db_path = test_db_path("empty_prevention");
    cleanup_db(&db_path);

    let mut app = App::new(&db_path).expect("Failed to create app");

    // Try to add empty todo
    app.input = "".to_string();
    app.add_todo().expect("Should handle empty input");
    assert!(app.todos.is_empty());

    // Try to add whitespace-only todo
    app.input = "   ".to_string();
    app.add_todo().expect("Should handle whitespace input");
    assert!(app.todos.is_empty());

    // Add valid todo
    app.input = "Valid todo".to_string();
    app.add_todo().expect("Failed to add valid todo");
    assert_eq!(app.todos.len(), 1);

    cleanup_db(&db_path);
}

#[test]
fn test_app_state_consistency() {
    let db_path = test_db_path("state_consistency");
    cleanup_db(&db_path);

    let mut app = App::new(&db_path).expect("Failed to create app");

    // Add todo
    app.input = "Test todo".to_string();
    app.add_todo().expect("Failed to add todo");

    // Verify state consistency
    assert_eq!(app.todos.len(), 1);
    assert!(app.input.is_empty(), "Input should be cleared after add");
    assert!(
        app.list_state.selected().is_some(),
        "Selection should exist"
    );

    // Toggle and verify
    app.toggle_selected().expect("Failed to toggle");
    assert!(app.todos[0].completed);

    // Verify refresh maintains consistency
    app.refresh_todos().expect("Failed to refresh");
    assert_eq!(app.todos.len(), 1);
    assert!(app.todos[0].completed);
    assert_eq!(app.list_state.selected(), Some(0));

    cleanup_db(&db_path);
}
