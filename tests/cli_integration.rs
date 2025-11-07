// Integration tests for CLI operations
// Tests the CLI binary by spawning processes and checking outputs

use std::fs;
use std::process::Command;

/// Helper to create a unique test database path
fn test_db_path(name: &str) -> String {
    format!("/tmp/test_cli_{}.db", name)
}

/// Helper to clean up test database
fn cleanup_db(path: &str) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(format!("{}-shm", path));
    let _ = fs::remove_file(format!("{}-wal", path));
}

/// Helper to run CLI commands
fn run_cli(db_path: &str, args: &[&str]) -> (bool, String, String) {
    let output = Command::new("cargo")
        .args(["run", "--bin", "todo-cli", "--"])
        .arg("--db-path")
        .arg(db_path)
        .args(args)
        .output()
        .expect("Failed to execute CLI command");

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (success, stdout, stderr)
}

#[test]
fn test_cli_add_todo() {
    let db_path = test_db_path("add");
    cleanup_db(&db_path);

    let (success, stdout, _) = run_cli(&db_path, &["add", "Test todo"]);
    assert!(success, "Add command should succeed");
    assert!(stdout.contains("Todo added successfully"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_add_empty_todo_fails() {
    let db_path = test_db_path("add_empty");
    cleanup_db(&db_path);

    let (success, _, stderr) = run_cli(&db_path, &["add", ""]);
    assert!(!success, "Add empty todo should fail");
    assert!(stderr.contains("Error"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_list_todos() {
    let db_path = test_db_path("list");
    cleanup_db(&db_path);

    // Add some todos
    run_cli(&db_path, &["add", "First todo"]);
    run_cli(&db_path, &["add", "Second todo"]);

    // List todos
    let (success, stdout, _) = run_cli(&db_path, &["list"]);
    assert!(success, "List command should succeed");
    assert!(stdout.contains("First todo"));
    assert!(stdout.contains("Second todo"));
    assert!(stdout.contains("ID"));
    assert!(stdout.contains("Status"));
    assert!(stdout.contains("Title"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_list_empty() {
    let db_path = test_db_path("list_empty");
    cleanup_db(&db_path);

    let (success, stdout, _) = run_cli(&db_path, &["list"]);
    assert!(success, "List command should succeed even when empty");
    assert!(stdout.contains("No todos found"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_toggle_todo() {
    let db_path = test_db_path("toggle");
    cleanup_db(&db_path);

    // Add a todo
    run_cli(&db_path, &["add", "Todo to toggle"]);

    // Toggle it to completed
    let (success, stdout, _) = run_cli(&db_path, &["toggle", "1"]);
    assert!(success, "Toggle command should succeed");
    assert!(stdout.contains("toggled successfully"));

    // List and verify it's completed (✓)
    let (_, list_stdout, _) = run_cli(&db_path, &["list"]);
    assert!(list_stdout.contains("✓"));

    // Toggle back
    run_cli(&db_path, &["toggle", "1"]);
    let (_, list_stdout2, _) = run_cli(&db_path, &["list"]);
    // When incomplete, the status shows [ ] instead of [✓]
    assert!(list_stdout2.contains("[ ]"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_toggle_nonexistent() {
    let db_path = test_db_path("toggle_nonexistent");
    cleanup_db(&db_path);

    let (success, _, stderr) = run_cli(&db_path, &["toggle", "999"]);
    assert!(!success, "Toggle nonexistent todo should fail");
    assert!(stderr.contains("Error"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_complete_todo() {
    let db_path = test_db_path("complete");
    cleanup_db(&db_path);

    // Add a todo
    run_cli(&db_path, &["add", "Todo to complete"]);

    // Complete it
    let (success, stdout, _) = run_cli(&db_path, &["complete", "1"]);
    assert!(success, "Complete command should succeed");
    assert!(stdout.contains("marked as complete"));

    // Verify it's completed
    let (_, list_stdout, _) = run_cli(&db_path, &["list"]);
    assert!(list_stdout.contains("✓"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_uncomplete_todo() {
    let db_path = test_db_path("uncomplete");
    cleanup_db(&db_path);

    // Add and complete a todo
    run_cli(&db_path, &["add", "Todo to uncomplete"]);
    run_cli(&db_path, &["complete", "1"]);

    // Uncomplete it
    let (success, stdout, _) = run_cli(&db_path, &["uncomplete", "1"]);
    assert!(success, "Uncomplete command should succeed");
    assert!(stdout.contains("marked as incomplete"));

    // Verify it's not completed
    let (_, list_stdout, _) = run_cli(&db_path, &["list"]);
    assert!(list_stdout.contains("[ ]"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_delete_todo() {
    let db_path = test_db_path("delete");
    cleanup_db(&db_path);

    // Add todos
    run_cli(&db_path, &["add", "First todo"]);
    run_cli(&db_path, &["add", "Second todo"]);

    // Delete first todo
    let (success, stdout, _) = run_cli(&db_path, &["delete", "1"]);
    assert!(success, "Delete command should succeed");
    assert!(stdout.contains("deleted successfully"));

    // Verify it's gone
    let (_, list_stdout, _) = run_cli(&db_path, &["list"]);
    assert!(!list_stdout.contains("First todo"));
    assert!(list_stdout.contains("Second todo"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_delete_nonexistent() {
    let db_path = test_db_path("delete_nonexistent");
    cleanup_db(&db_path);

    let (success, _, stderr) = run_cli(&db_path, &["delete", "999"]);
    assert!(!success, "Delete nonexistent todo should fail");
    assert!(stderr.contains("Error"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_get_todo() {
    let db_path = test_db_path("get");
    cleanup_db(&db_path);

    // Add a todo
    run_cli(&db_path, &["add", "Todo to retrieve"]);

    // Get the todo
    let (success, stdout, _) = run_cli(&db_path, &["get", "1"]);
    assert!(success, "Get command should succeed");
    assert!(stdout.contains("ID: 1"));
    assert!(stdout.contains("Title: Todo to retrieve"));
    assert!(stdout.contains("Status: incomplete"));

    // Complete it and get again
    run_cli(&db_path, &["complete", "1"]);
    let (_, stdout2, _) = run_cli(&db_path, &["get", "1"]);
    assert!(stdout2.contains("Status: completed"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_get_nonexistent() {
    let db_path = test_db_path("get_nonexistent");
    cleanup_db(&db_path);

    let (success, _, stderr) = run_cli(&db_path, &["get", "999"]);
    assert!(!success, "Get nonexistent todo should fail");
    assert!(stderr.contains("not found") || stderr.contains("Error"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_update_todo() {
    let db_path = test_db_path("update");
    cleanup_db(&db_path);

    // Add a todo
    run_cli(&db_path, &["add", "Original title"]);

    // Update it
    let (success, stdout, _) = run_cli(&db_path, &["update", "1", "Updated title"]);
    assert!(success, "Update command should succeed");
    assert!(stdout.contains("updated successfully"));

    // Verify the update
    let (_, list_stdout, _) = run_cli(&db_path, &["list"]);
    assert!(list_stdout.contains("Updated title"));
    assert!(!list_stdout.contains("Original title"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_update_empty_title_fails() {
    let db_path = test_db_path("update_empty");
    cleanup_db(&db_path);

    // Add a todo
    run_cli(&db_path, &["add", "Original title"]);

    // Try to update with empty title
    let (success, _, stderr) = run_cli(&db_path, &["update", "1", ""]);
    assert!(!success, "Update with empty title should fail");
    assert!(stderr.contains("Error"));

    // Verify the title is unchanged
    let (_, list_stdout, _) = run_cli(&db_path, &["list"]);
    assert!(list_stdout.contains("Original title"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_update_nonexistent() {
    let db_path = test_db_path("update_nonexistent");
    cleanup_db(&db_path);

    let (success, _, stderr) = run_cli(&db_path, &["update", "999", "New title"]);
    assert!(!success, "Update nonexistent todo should fail");
    assert!(stderr.contains("Error"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_workflow_add_complete_delete() {
    let db_path = test_db_path("workflow");
    cleanup_db(&db_path);

    // Add multiple todos
    run_cli(&db_path, &["add", "Buy groceries"]);
    run_cli(&db_path, &["add", "Clean house"]);
    run_cli(&db_path, &["add", "Write code"]);

    // List all
    let (_, list_stdout, _) = run_cli(&db_path, &["list"]);
    assert!(list_stdout.contains("Buy groceries"));
    assert!(list_stdout.contains("Clean house"));
    assert!(list_stdout.contains("Write code"));

    // Complete one
    run_cli(&db_path, &["complete", "2"]);

    // Delete one
    run_cli(&db_path, &["delete", "1"]);

    // Verify final state
    let (_, final_list, _) = run_cli(&db_path, &["list"]);
    assert!(!final_list.contains("Buy groceries"));
    assert!(final_list.contains("Clean house"));
    assert!(final_list.contains("✓")); // Completed indicator
    assert!(final_list.contains("Write code"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_update_preserves_completion() {
    let db_path = test_db_path("update_preserves");
    cleanup_db(&db_path);

    // Add and complete a todo
    run_cli(&db_path, &["add", "Original title"]);
    run_cli(&db_path, &["complete", "1"]);

    // Update the title
    run_cli(&db_path, &["update", "1", "New title"]);

    // Verify both title changed and completion status preserved
    let (_, list_stdout, _) = run_cli(&db_path, &["list"]);
    assert!(list_stdout.contains("New title"));
    assert!(list_stdout.contains("✓"));

    cleanup_db(&db_path);
}

#[test]
fn test_cli_help_command() {
    // Test that --help works
    let output = Command::new("cargo")
        .args(["run", "--bin", "todo-cli", "--", "--help"])
        .output()
        .expect("Failed to execute CLI command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("todo-cli"));
    assert!(stdout.contains("Commands:"));
    assert!(stdout.contains("add"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("toggle"));
    assert!(stdout.contains("delete"));
}

#[test]
fn test_cli_with_custom_db_path() {
    let custom_path = "/tmp/custom_cli_test.db";
    cleanup_db(custom_path);

    // Use environment variable to set custom path
    let output = Command::new("cargo")
        .args(["run", "--bin", "todo-cli", "--"])
        .env("TODO_DB_PATH", custom_path)
        .args(["add", "Custom path todo"])
        .output()
        .expect("Failed to execute CLI command");

    assert!(output.status.success());

    // Verify the file was created at the custom path
    assert!(std::path::Path::new(custom_path).exists());

    cleanup_db(custom_path);
}
