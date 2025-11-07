use clap::{Parser, Subcommand};
use rust_ratatui_todo::db::Database;
use std::process;

#[derive(Parser)]
#[command(name = "todo-cli")]
#[command(about = "A command-line todo list manager", long_about = None)]
struct Cli {
    /// Path to the database file
    #[arg(short, long, default_value = "./tmp/todos.db", env = "TODO_DB_PATH")]
    db_path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new todo
    Add {
        /// The title of the todo
        #[arg(value_name = "TITLE")]
        title: String,
    },
    /// List all todos
    List {
        /// Page number (0-indexed)
        #[arg(long, default_value = "0")]
        page: u32,
        /// Number of items per page
        #[arg(long, default_value = "20")]
        page_size: u32,
    },
    /// Toggle completion status of a todo
    Toggle {
        /// The ID of the todo to toggle
        #[arg(value_name = "ID")]
        id: i64,
    },
    /// Mark a todo as complete
    Complete {
        /// The ID of the todo to complete
        #[arg(value_name = "ID")]
        id: i64,
    },
    /// Mark a todo as incomplete
    Uncomplete {
        /// The ID of the todo to mark as incomplete
        #[arg(value_name = "ID")]
        id: i64,
    },
    /// Delete a todo
    Delete {
        /// The ID of the todo to delete
        #[arg(value_name = "ID")]
        id: i64,
    },
    /// Get a specific todo by ID
    Get {
        /// The ID of the todo to retrieve
        #[arg(value_name = "ID")]
        id: i64,
    },
    /// Update a todo's title
    Update {
        /// The ID of the todo to update
        #[arg(value_name = "ID")]
        id: i64,
        /// The new title
        #[arg(value_name = "TITLE")]
        title: String,
    },
    /// Clear all todos (requires confirmation)
    Clear {
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // Create database connection
    let db = match Database::new(&cli.db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error: Failed to open database: {}", e);
            process::exit(1);
        }
    };

    // Execute the command
    let result = match cli.command {
        Commands::Add { title } => add_todo(&db, &title),
        Commands::List { page, page_size } => list_todos(&db, page, page_size),
        Commands::Toggle { id } => toggle_todo(&db, id),
        Commands::Complete { id } => complete_todo(&db, id),
        Commands::Uncomplete { id } => uncomplete_todo(&db, id),
        Commands::Delete { id } => delete_todo(&db, id),
        Commands::Get { id } => get_todo(&db, id),
        Commands::Update { id, title } => update_todo(&db, id, &title),
        Commands::Clear { yes } => clear_all(&db, yes),
    };

    // Handle errors
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn add_todo(db: &Database, title: &str) -> Result<(), String> {
    db.add_todo(title)
        .map_err(|e| format!("Failed to add todo: {}", e))?;
    println!("Todo added successfully");
    Ok(())
}

fn list_todos(db: &Database, page: u32, page_size: u32) -> Result<(), String> {
    let total_count = db
        .count_todos()
        .map_err(|e| format!("Failed to count todos: {}", e))?;

    let todos = db
        .get_todos_paginated(page, page_size)
        .map_err(|e| format!("Failed to list todos: {}", e))?;

    let total_pages = if total_count == 0 {
        1
    } else {
        total_count.div_ceil(page_size)
    };

    println!(
        "Page {}/{} | Total: {} | Showing: {}",
        page + 1,
        total_pages,
        total_count,
        todos.len()
    );
    println!();

    if todos.is_empty() {
        println!("No todos found on this page");
        return Ok(());
    }

    println!("ID  Status  Title");
    println!("{}  {}  {}", "─".repeat(4), "─".repeat(6), "─".repeat(30));

    for todo in todos {
        let status = if todo.completed { "✓" } else { " " };
        println!("{:<4} [{}]     {}", todo.id, status, todo.title);
    }

    Ok(())
}

fn toggle_todo(db: &Database, id: i64) -> Result<(), String> {
    db.toggle_todo(id)
        .map_err(|e| format!("Failed to toggle todo: {}", e))?;
    println!("Todo {} toggled successfully", id);
    Ok(())
}

fn complete_todo(db: &Database, id: i64) -> Result<(), String> {
    // First check if the todo exists and get its status
    let todo = db
        .get_todo_by_id(id)
        .map_err(|e| format!("Failed to get todo: {}", e))?
        .ok_or_else(|| format!("Todo with ID {} not found", id))?;

    if !todo.completed {
        db.toggle_todo(id)
            .map_err(|e| format!("Failed to complete todo: {}", e))?;
    }

    println!("Todo {} marked as complete", id);
    Ok(())
}

fn uncomplete_todo(db: &Database, id: i64) -> Result<(), String> {
    // First check if the todo exists and get its status
    let todo = db
        .get_todo_by_id(id)
        .map_err(|e| format!("Failed to get todo: {}", e))?
        .ok_or_else(|| format!("Todo with ID {} not found", id))?;

    if todo.completed {
        db.toggle_todo(id)
            .map_err(|e| format!("Failed to uncomplete todo: {}", e))?;
    }

    println!("Todo {} marked as incomplete", id);
    Ok(())
}

fn delete_todo(db: &Database, id: i64) -> Result<(), String> {
    db.delete_todo(id)
        .map_err(|e| format!("Failed to delete todo: {}", e))?;
    println!("Todo {} deleted successfully", id);
    Ok(())
}

fn get_todo(db: &Database, id: i64) -> Result<(), String> {
    let todo = db
        .get_todo_by_id(id)
        .map_err(|e| format!("Failed to get todo: {}", e))?
        .ok_or_else(|| format!("Todo with ID {} not found", id))?;

    let status = if todo.completed {
        "completed"
    } else {
        "incomplete"
    };

    println!("ID: {}", todo.id);
    println!("Title: {}", todo.title);
    println!("Status: {}", status);

    Ok(())
}

fn update_todo(db: &Database, id: i64, title: &str) -> Result<(), String> {
    db.update_todo_title(id, title)
        .map_err(|e| format!("Failed to update todo: {}", e))?;
    println!("Todo {} updated successfully", id);
    Ok(())
}

fn clear_all(db: &Database, skip_confirmation: bool) -> Result<(), String> {
    if !skip_confirmation {
        println!("WARNING: This will delete all todos!");
        println!("Type 'yes' to confirm:");

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Failed to read input: {}", e))?;

        if input.trim().to_lowercase() != "yes" {
            println!("Operation cancelled");
            return Ok(());
        }
    }

    let count = db
        .clear_all()
        .map_err(|e| format!("Failed to clear todos: {}", e))?;

    println!("Successfully deleted {} todos", count);
    Ok(())
}
