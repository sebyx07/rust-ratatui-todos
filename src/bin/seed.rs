use rust_ratatui_todo::db::Database;
use std::process;

fn main() {
    // Parse arguments
    let args: Vec<String> = std::env::args().collect();

    let count: usize = if args.len() > 1 {
        args[1].parse().unwrap_or_else(|_| {
            eprintln!("Error: Invalid count. Please provide a positive number.");
            process::exit(1);
        })
    } else {
        10_000 // Default to 10k
    };

    let db_path = std::env::var("TODO_DB_PATH").unwrap_or_else(|_| "./tmp/todos.db".to_string());

    // Ensure the directory exists
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|e| {
            eprintln!("Error: Failed to create directory: {}", e);
            process::exit(1);
        });
    }

    // Create database connection
    let db = Database::new(&db_path).unwrap_or_else(|e| {
        eprintln!("Error: Failed to open database: {}", e);
        process::exit(1);
    });

    println!("Seeding {} todos to database: {}", count, db_path);
    println!("This may take a moment...\n");

    let start_time = std::time::Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;

    // Progress reporting intervals
    let report_interval = count / 10; // Report every 10%

    for i in 1..=count {
        let title = generate_todo_title(i);

        match db.add_todo(&title) {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                if error_count == 1 {
                    // Only print first error to avoid spam
                    eprintln!("Warning: Error adding todo #{}: {}", i, e);
                }
            }
        }

        // Progress reporting
        if report_interval > 0 && i % report_interval == 0 {
            let percentage = (i as f64 / count as f64) * 100.0;
            println!("Progress: {:.0}% ({}/{})", percentage, i, count);
        }
    }

    let elapsed = start_time.elapsed();

    println!("\n=== Seeding Complete ===");
    println!("Successfully added: {} todos", success_count);

    if error_count > 0 {
        println!("Errors encountered: {} todos", error_count);
    }

    println!("Time elapsed: {:.2}s", elapsed.as_secs_f64());
    println!(
        "Average rate: {:.0} todos/sec",
        success_count as f64 / elapsed.as_secs_f64()
    );
}

/// Generate a varied todo title based on the index
fn generate_todo_title(index: usize) -> String {
    let templates = vec![
        format!("Complete project task #{}", index),
        format!("Review pull request #{}", index),
        format!("Fix bug in module {}", index),
        format!("Update documentation for feature {}", index),
        format!("Test new functionality #{}", index),
        format!("Refactor code in component {}", index),
        format!("Write unit tests for function {}", index),
        format!("Optimize performance in section {}", index),
        format!("Design UI mockup for page {}", index),
        format!("Schedule meeting about topic {}", index),
        format!("Research technology option {}", index),
        format!("Deploy version 1.{}.0", index % 100),
        format!("Investigate issue #{}", index),
        format!("Implement feature request {}", index),
        format!("Create database migration {}", index),
        format!("Update dependencies for package {}", index),
        format!("Review security audit item {}", index),
        format!("Configure CI/CD pipeline stage {}", index),
        format!("Analyze metrics for dashboard {}", index),
        format!("Plan sprint tasks for week {}", index % 52 + 1),
    ];

    // Use index to deterministically select a template
    templates[index % templates.len()].clone()
}
