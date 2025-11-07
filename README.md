# Rust Ratatui Todo List

A terminal-based todo list application built with Rust, featuring persistent SQLite storage and an intuitive TUI powered by Ratatui.

## Features

- **Persistent Storage**: All todos are saved to a SQLite database
- **Terminal UI**: Clean, intuitive interface using Ratatui
- **Vim-Style Navigation**: Efficient keyboard controls
- **CRUD Operations**: Create, read, update (toggle), and delete todos
- **Comprehensive Tests**: 19 unit tests with full coverage
- **Clean Architecture**: Modular code with separation of concerns

## Project Structure

```
src/
├── main.rs    # Entry point and event loop
├── app.rs     # Application state and business logic
├── db.rs      # SQLite database operations
└── ui.rs      # UI rendering components
```

## Installation

### Prerequisites

- Rust 1.75 or higher
- Cargo

### Build

```bash
cargo build --release
```

## Usage

Run the application:

```bash
cargo run --release
```

### Keyboard Controls

#### Normal Mode
- `q` - Quit the application
- `i` or `a` - Enter editing mode to add a new todo
- `j` or `↓` - Move down in the list
- `k` or `↑` - Move up in the list
- `Space` - Toggle todo completion status
- `d` or `Delete` - Delete the selected todo

#### Editing Mode
- Type to enter todo text
- `Enter` - Save the new todo
- `Esc` - Cancel and return to normal mode
- `Backspace` - Delete last character

## Data Storage

Todos are stored in `./tmp/todos.db`. The database is created automatically on first run.

## Development

### Running Tests

```bash
cargo test
```

All 19 tests should pass:
- 12 database layer tests
- 7 application logic tests

### Code Quality

The codebase passes all Clippy lints with warnings treated as errors:

```bash
cargo clippy -- -D warnings
```

### Test Coverage

- Database initialization and schema creation
- CRUD operations (Create, Read, Update, Delete)
- Input validation and error handling
- Navigation and selection logic
- State management and mode transitions
- Edge cases (empty lists, invalid IDs, etc.)

## Architecture

### Database Layer (`db.rs`)
- SQLite connection management
- Schema initialization
- CRUD operations with proper error handling
- Input validation

### Application Layer (`app.rs`)
- Application state management
- Business logic for todo operations
- Navigation and input handling
- Mode transitions (Normal/Editing)

### UI Layer (`ui.rs`)
- Rendering todo list
- Status bar and help text
- Visual styling and colors

### Main (`main.rs`)
- Terminal initialization and cleanup
- Event loop
- Keyboard event handling

## Dependencies

- `ratatui` (0.29) - Terminal UI framework
- `crossterm` (0.28) - Cross-platform terminal manipulation
- `rusqlite` (0.32) - SQLite database bindings

## License

This project was created as a demonstration of Rust TUI development with Ratatui.
