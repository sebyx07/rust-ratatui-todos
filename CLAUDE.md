# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Testing
```bash
# Run all tests (67 tests: 23 unit + 44 integration)
cargo test

# Run specific test file
cargo test --test database_integration
cargo test --test app_workflow
cargo test --test server_api
cargo test --test cli_integration

# Run tests in a specific module
cargo test db::tests::
cargo test app::tests::

# Run a single test by name
cargo test test_add_todo
```

### Building and Running

#### TUI Client
```bash
# Build debug version
cargo build --bin todo-tui

# Build release version (recommended for performance)
cargo build --release --bin todo-tui

# Run the TUI application
cargo run --release --bin todo-tui
```

#### HTTP Server
```bash
# Build the server
cargo build --bin todo-server

# Run the server (default port: 3000, default DB: ./tmp/todos.db)
cargo run --bin todo-server

# Run with custom port and database path
PORT=8080 TODO_DB_PATH=/tmp/my-todos.db cargo run --bin todo-server
```

The server provides a REST API with the following endpoints:
- `GET /health` - Health check
- `GET /todos` - List all todos
- `GET /todos/:id` - Get a specific todo
- `POST /todos` - Create a new todo (body: `{"title": "..."}`)
- `PUT /todos/:id` - Update a todo (body: `{"title": "...", "completed": true/false}`)
- `DELETE /todos/:id` - Delete a todo

Example usage:
```bash
# Create a todo
curl -X POST http://localhost:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy groceries"}'

# List all todos
curl http://localhost:3000/todos

# Update a todo
curl -X PUT http://localhost:3000/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"completed":true}'

# Delete a todo
curl -X DELETE http://localhost:3000/todos/1
```

#### CLI Client
```bash
# Build the CLI
cargo build --bin todo-cli

# Run the CLI (default DB: ./tmp/todos.db)
cargo run --bin todo-cli -- <command>

# Use custom database path
cargo run --bin todo-cli -- --db-path /tmp/my-todos.db <command>

# Or via environment variable
TODO_DB_PATH=/tmp/my-todos.db cargo run --bin todo-cli -- <command>
```

The CLI provides the following commands:

**Add a new todo:**
```bash
cargo run --bin todo-cli -- add "Buy groceries"
```

**List all todos:**
```bash
cargo run --bin todo-cli -- list
```

**Toggle completion status:**
```bash
cargo run --bin todo-cli -- toggle 1
```

**Mark as complete:**
```bash
cargo run --bin todo-cli -- complete 1
```

**Mark as incomplete:**
```bash
cargo run --bin todo-cli -- uncomplete 1
```

**Get a specific todo:**
```bash
cargo run --bin todo-cli -- get 1
```

**Update todo title:**
```bash
cargo run --bin todo-cli -- update 1 "New title"
```

**Delete a todo:**
```bash
cargo run --bin todo-cli -- delete 1
```

**Get help:**
```bash
cargo run --bin todo-cli -- --help
```

#### General
```bash
# Clean build artifacts
cargo clean
```

### Linting
```bash
# Run Clippy with warnings as errors (project standard)
cargo clippy -- -D warnings

# Apply Clippy suggestions automatically
cargo clippy --fix
```

### Code Formatting
```bash
# Check formatting
cargo fmt -- --check

# Format code
cargo fmt
```

## Architecture Overview

This project contains three executables that share the same database layer:
1. **TUI Client** (`main.rs`) - Terminal-based interactive todo list application
2. **HTTP Server** (`src/bin/server.rs`) - REST API server for managing todos
3. **CLI Client** (`src/bin/cli.rs`) - Command-line interface for scripting and quick operations

The TUI client uses a clean 3-layer architecture following separation of concerns, while the server and CLI directly use the database layer:

### Layer 1: Database Layer (`db.rs`)
- **Purpose**: SQLite persistence and data access
- **Key type**: `Database` struct wraps `rusqlite::Connection`
- **Key type**: `Todo` struct represents data model
- **Operations**: CRUD operations with validation
- **Testing**: Uses `:memory:` databases for unit tests
- **Error handling**: Returns `rusqlite::Result` for all operations
- **Validation**: Validates empty titles, checks row counts for updates/deletes

### Layer 2: Application Layer (`app.rs`)
- **Purpose**: Business logic and state management
- **Key type**: `App` struct owns `Database`, todo list, UI state
- **State management**:
  - `InputMode` enum: `Normal` vs `Editing` modes
  - `ListState` tracks selected todo (from ratatui)
  - `input` String buffer for new todo text
- **Operations**: Wraps database operations with UI state updates
- **Key pattern**: Operations call `refresh_todos()` to sync state after mutations
- **Selection handling**: Automatically adjusts selection when todos deleted or list becomes empty

### Layer 3: UI Layer (`ui.rs`)
- **Purpose**: Rendering using Ratatui framework
- **Layout**: 3-part vertical layout (title bar, todo list, status bar)
- **Rendering functions**:
  - `render()`: Main entry point, splits screen into chunks
  - `render_title()`: Static cyan title bar
  - `render_todo_list()`: List with checkboxes, strikethrough for completed
  - `render_status_bar()`: Context-sensitive help text based on InputMode
- **Styling**: Completed todos shown in gray with strikethrough, selected todos highlighted

### Entry Point - TUI Client (`main.rs`)
- **Terminal setup**: Uses Crossterm for terminal manipulation
- **Initialization**: Creates `./tmp/todos.db` on startup
- **Event loop**:
  - Reads keyboard events with `crossterm::event::read()`
  - Filters for `KeyEventKind::Press` to avoid duplicate events
  - Dispatches to different handlers based on `InputMode`
- **Cleanup**: Restores terminal state in defer-like pattern with `Result` return

### HTTP Server (`src/bin/server.rs`)
- **Framework**: Built with Axum web framework on Tokio async runtime
- **API Design**: RESTful JSON API with proper HTTP status codes
- **State Management**: Uses `Arc<Mutex<Database>>` for thread-safe database access
- **Endpoints**:
  - Health check for monitoring
  - Full CRUD operations on todos
  - Proper validation and error responses
- **Configuration**: Port and database path configurable via environment variables
- **CORS**: Permissive CORS policy enabled for development
- **Error Handling**: Custom `ApiError` enum with appropriate HTTP status codes

### CLI Client (`src/bin/cli.rs`)
- **Framework**: Built with Clap for command-line argument parsing
- **Commands**: Full CRUD operations via subcommands (add, list, toggle, complete, uncomplete, delete, get, update)
- **Database Access**: Direct use of `Database` layer, same as server
- **Configuration**: Database path configurable via `--db-path` flag or `TODO_DB_PATH` environment variable
- **Output**: Human-readable formatted output for list and get commands
- **Error Handling**: Returns appropriate exit codes (0 for success, 1 for errors)
- **Use Cases**: Scripting, quick operations, batch processing, CI/CD integration

### Testing Architecture (`lib.rs` + `tests/`)
- **Library crate**: `lib.rs` exposes modules for integration testing
- **Unit tests**: Colocated with implementation using `#[cfg(test)]` modules (23 tests)
- **Integration tests**: Separate `tests/` directory with four test files (44 tests):
  - `database_integration.rs`: Tests persistence, concurrency, SQL injection prevention, large datasets
  - `app_workflow.rs`: Tests end-to-end workflows, mode transitions, selection logic
  - `server_api.rs`: Tests server database operations, thread safety, JSON serialization, validation
  - `cli_integration.rs`: Tests CLI commands via process spawning, exit codes, output validation, workflows

## Key Design Patterns

### Database Connection Management
- Single connection per `App` instance (no connection pooling needed)
- In-memory databases (`:memory:`) used for all tests
- SQLite bundled via `rusqlite` features, no external dependencies

### Error Handling
- Database layer returns `rusqlite::Result`
- Application layer wraps in `Box<dyn Error>` for flexibility
- Validation at database layer (empty titles rejected)
- Operation validation (row count checks for updates/deletes)

### State Synchronization
- After any mutation (add/toggle/delete), call `refresh_todos()` to reload from database
- `refresh_todos()` also adjusts selection bounds when list changes
- Selection logic handles wraparound (next from end goes to start, previous from start goes to end)

### Mode-Based Input Handling
Two distinct input modes with different key bindings:
- **Normal mode**: Navigation (j/k/arrows), actions (space/d/delete), mode switch (i/a)
- **Editing mode**: Text input (chars), completion (Enter), cancellation (Esc)

### Testing with In-Memory Databases
All tests use `:memory:` SQLite databases for:
- Fast test execution
- No file system cleanup needed
- Parallel test execution without conflicts
- Idempotent tests with fresh state

## Development Guidelines

### Adding New Database Operations
1. Add method to `Database` in `db.rs`
2. Return `rusqlite::Result` type
3. Validate inputs and check row counts for mutations
4. Add unit test in `db.rs` tests module
5. Add integration test in `tests/database_integration.rs` if testing persistence/concurrency

### Adding New Application Features
1. Add method to `App` in `app.rs`
2. Return `Result<(), Box<dyn Error>>` for operations that can fail
3. Call `refresh_todos()` after any database mutations
4. Add unit test in `app.rs` tests module
5. Add integration test in `tests/app_workflow.rs` for multi-step workflows

### Adding UI Features
1. Modify rendering functions in `ui.rs` (pure functions, no state)
2. UI functions take `&mut Frame` and `&App` (or `&mut App` for stateful widgets)
3. Use Ratatui layout constraints for responsive design
4. Test UI changes by running the application, no automated UI tests

### Adding Keyboard Shortcuts
1. Add key handling in `main.rs` event loop
2. Match on `key.code` in appropriate `InputMode` branch
3. Call corresponding `App` method
4. Update status bar help text in `ui.rs` `render_status_bar()`

### Performance Considerations
- SQLite queries use prepared statements (automatic with `rusqlite`)
- Todos loaded fully into memory on each refresh (acceptable for typical todo list sizes)
- For large datasets (1000+ todos), consider pagination in UI layer
- Event loop blocks on keyboard input (no busy waiting)

### Concurrency Notes
- Application is single-threaded (event loop on main thread)
- SQLite connection not thread-safe, no `Send`/`Sync` needed
- Integration tests verify concurrent operations by creating multiple `Database` instances
- No async runtime needed (blocking I/O acceptable for this use case)

### Common Pitfalls
- **Don't forget `refresh_todos()`**: After mutations, always sync state
- **Handle empty lists**: Check `todos.is_empty()` before accessing by index
- **Test both modes**: Verify features work in both Normal and Editing modes
- **Selection bounds**: When modifying list, ensure selection stays valid
- **Key event filtering**: Only handle `KeyEventKind::Press` to avoid duplicate events

### Rust Edition and Features
- Uses Rust 2024 edition
- Requires unstable features for let-chains in if conditions (`if let && let`)
- Build with nightly or stable Rust 1.76+ (let-chains stabilized in 1.76)
