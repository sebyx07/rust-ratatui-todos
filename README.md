# 📝 Rust Ratatui Todo List

<div align="center">

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![SQLite](https://img.shields.io/badge/sqlite-%2307405e.svg?style=for-the-badge&logo=sqlite&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)

**A powerful, feature-rich todo list application with multiple interfaces**

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Architecture](#-architecture) • [Development](#-development)

</div>

---

## ✨ Features

### 🎯 Multiple Interfaces
- **🖥️ TUI (Terminal User Interface)**: Beautiful, interactive terminal UI with vim-style navigation
- **⚡ CLI**: Command-line interface perfect for scripting and automation
- **🌐 REST API Server**: HTTP server with full CRUD operations

### 💾 Robust Data Management
- **Persistent Storage**: SQLite database with automatic schema management
- **📄 Pagination**: Handle 100k+ todos effortlessly with built-in pagination
- **🔒 Data Integrity**: Full ACID compliance with SQLite transactions
- **🧹 Bulk Operations**: Clear all todos with confirmation

### 🎨 Developer Experience
- **✅ Comprehensive Testing**: 70+ tests (26 unit + 44 integration)
- **📚 Clean Architecture**: Modular design with separation of concerns
- **🚀 High Performance**: Efficient pagination and database indexing
- **🔧 Easy Configuration**: Environment variables and CLI flags

### 🧪 Testing & Quality
- **Integration Tests**: Database, API, CLI, and workflow tests
- **Clippy Clean**: Zero warnings with `clippy -- -D warnings`
- **Formatted**: Consistent formatting with `rustfmt`
- **Seeding Tool**: Generate test data with 10k+ todos instantly

---

## 📁 Project Structure

```
rust-ratatui-todo/
├── src/
│   ├── lib.rs                    # Library crate exports
│   ├── main.rs                   # TUI entry point
│   ├── app.rs                    # Application state & pagination logic
│   ├── db.rs                     # Database layer with pagination
│   ├── ui.rs                     # TUI rendering components
│   ├── event_handler.rs          # Keyboard event handling
│   ├── terminal.rs               # Terminal setup/cleanup
│   ├── models/                   # Data models
│   │   ├── mod.rs
│   │   ├── todo.rs               # Todo struct
│   │   └── input_mode.rs         # Input mode enum
│   └── bin/
│       ├── cli.rs                # CLI interface
│       ├── server.rs             # HTTP server
│       └── seed.rs               # Database seeding tool
├── tests/
│   ├── database_integration.rs   # Database & persistence tests
│   ├── app_workflow.rs           # App workflow tests
│   ├── server_api.rs             # Server API tests
│   └── cli_integration.rs        # CLI integration tests
├── CLAUDE.md                     # Development guidelines
├── Cargo.toml
└── README.md
```

---

## 🚀 Installation

### Prerequisites

- **Rust**: 1.76 or higher (for let-chains support)
- **Cargo**: Comes with Rust

### Quick Start

```bash
# Clone the repository
git clone https://github.com/yourusername/rust-ratatui-todo.git
cd rust-ratatui-todo

# Build all binaries
cargo build --release

# Or build individually
cargo build --release --bin todo-tui
cargo build --release --bin todo-cli
cargo build --release --bin todo-server
cargo build --release --bin seed-todos
```

---

## 📖 Usage

### 🖥️ TUI (Terminal User Interface)

```bash
cargo run --release --bin todo-tui
```

#### Keyboard Controls

**Normal Mode:**
- `q` - Quit
- `i`/`a` - Add new todo
- `j`/`↓` - Move down
- `k`/`↑` - Move up
- `Space` - Toggle completion
- `d`/`Delete` - Delete selected
- `n`/`PageDown` - Next page
- `p`/`PageUp` - Previous page
- `Shift+C` - Clear all (with confirmation)

**Editing Mode:**
- Type to enter text
- `Enter` - Save
- `Esc` - Cancel
- `Backspace` - Delete character

---

### ⚡ CLI (Command Line Interface)

```bash
# Add a new todo
cargo run --bin todo-cli -- add "Buy groceries"

# List todos (with pagination)
cargo run --bin todo-cli -- list
cargo run --bin todo-cli -- list --page 0 --page-size 20

# Get a specific todo
cargo run --bin todo-cli -- get 1

# Toggle completion
cargo run --bin todo-cli -- toggle 1

# Mark as complete/incomplete
cargo run --bin todo-cli -- complete 1
cargo run --bin todo-cli -- uncomplete 1

# Update todo title
cargo run --bin todo-cli -- update 1 "New title"

# Delete a todo
cargo run --bin todo-cli -- delete 1

# Clear all todos
cargo run --bin todo-cli -- clear
cargo run --bin todo-cli -- clear --yes  # Skip confirmation

# Use custom database path
cargo run --bin todo-cli -- --db-path /tmp/my-todos.db list
# Or via environment variable
TODO_DB_PATH=/tmp/my-todos.db cargo run --bin todo-cli -- list

# Get help
cargo run --bin todo-cli -- --help
```

---

### 🌐 REST API Server

```bash
# Start the server (default: http://localhost:3000)
cargo run --bin todo-server

# Custom port and database
PORT=8080 TODO_DB_PATH=/tmp/todos.db cargo run --bin todo-server
```

#### API Endpoints

**GET /health** - Health check
```bash
curl http://localhost:3000/health
```

**GET /todos** - List todos (with pagination)
```bash
# Get first page (default page_size=20)
curl http://localhost:3000/todos

# With pagination parameters
curl "http://localhost:3000/todos?page=0&page_size=50"

# Response format:
{
  "todos": [...],
  "page": 0,
  "page_size": 20,
  "total_count": 150,
  "total_pages": 8
}
```

**GET /todos/:id** - Get specific todo
```bash
curl http://localhost:3000/todos/1
```

**POST /todos** - Create new todo
```bash
curl -X POST http://localhost:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"title":"Buy groceries"}'
```

**PUT /todos/:id** - Update todo
```bash
# Update title
curl -X PUT http://localhost:3000/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"title":"New title"}'

# Toggle completion
curl -X PUT http://localhost:3000/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"completed":true}'

# Update both
curl -X PUT http://localhost:3000/todos/1 \
  -H "Content-Type: application/json" \
  -d '{"title":"New title","completed":true}'
```

**DELETE /todos/:id** - Delete todo
```bash
curl -X DELETE http://localhost:3000/todos/1
```

---

### 🌱 Database Seeding

Generate test data for testing pagination and performance:

```bash
# Seed 10,000 todos (default)
cargo run --bin seed-todos

# Custom count
cargo run --bin seed-todos 100000

# Custom database
TODO_DB_PATH=/tmp/test.db cargo run --bin seed-todos 50000
```

Example output:
```
Seeding 10000 todos to database: ./tmp/todos.db
This may take a moment...

Progress: 10% (1000/10000)
Progress: 20% (2000/10000)
...
Progress: 100% (10000/10000)

=== Seeding Complete ===
Successfully added: 10000 todos
Time elapsed: 2.34s
Average rate: 4274 todos/sec
```

---

## 🏗️ Architecture

### Layer 1: Database Layer (`db.rs`)
- **Purpose**: SQLite persistence and data access
- **Features**: CRUD operations, pagination, validation
- **Key Methods**:
  - `get_todos()` - Get all todos
  - `get_todos_paginated(page, page_size)` - Get paginated results
  - `count_todos()` - Get total count
  - `add_todo()`, `toggle_todo()`, `delete_todo()`, `update_todo_title()`
  - `clear_all()` - Bulk delete operation

### Layer 2: Application Layer (`app.rs`)
- **Purpose**: Business logic and state management
- **Features**: Pagination state, UI state, input handling
- **Key Methods**:
  - `next_page()`, `previous_page()` - Page navigation
  - `pagination_info()` - Display pagination stats
  - `clear_all()` - Clear with state refresh

### Layer 3: UI Layer (`ui.rs`)
- **Purpose**: TUI rendering with Ratatui
- **Features**: Multi-panel layout, pagination display
- **Components**:
  - Title bar
  - Todo list (with checkboxes and strikethrough)
  - Pagination info bar
  - Status/controls bar

### Executables

1. **TUI** (`main.rs`) - Interactive terminal interface
2. **CLI** (`bin/cli.rs`) - Command-line tool with Clap
3. **Server** (`bin/server.rs`) - Axum REST API with Tokio
4. **Seeder** (`bin/seed.rs`) - Bulk data generation tool

---

## 🧪 Development

### Running Tests

```bash
# Run all tests (70+ tests)
cargo test

# Run specific test suite
cargo test --test database_integration
cargo test --test app_workflow
cargo test --test server_api
cargo test --test cli_integration

# Run unit tests only
cargo test --lib

# Run with output
cargo test -- --nocapture
```

#### Test Coverage
- **26 Unit Tests**: Database operations, app logic, pagination
- **6 Database Integration Tests**: Persistence, concurrency, special characters
- **9 App Workflow Tests**: End-to-end workflows, state management
- **10 Server API Tests**: HTTP endpoints, JSON serialization, thread safety
- **19 CLI Integration Tests**: Command execution, exit codes, output validation

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run Clippy
cargo clippy -- -D warnings

# Apply Clippy fixes
cargo clippy --fix
```

### Building for Production

```bash
# Optimized build
cargo build --release

# Binaries will be in target/release/:
# - todo-tui
# - todo-cli
# - todo-server
# - seed-todos
```

---

## 📊 Performance

### Pagination Performance
- **100k todos**: Smooth navigation with 50 items/page
- **Database queries**: ~1-2ms per page load with SQLite LIMIT/OFFSET
- **Memory efficient**: Only loads current page into memory

### Seeding Benchmarks
- **10k todos**: ~2-3 seconds
- **100k todos**: ~20-25 seconds
- **Average rate**: ~4000-5000 todos/second

---

## 🔧 Configuration

### Environment Variables

- `TODO_DB_PATH` - Database file path (default: `./tmp/todos.db`)
- `PORT` - Server port (default: `3000`)

### Default Values

- **TUI page size**: 50 items
- **CLI page size**: 20 items
- **Server page size**: 20 items
- **Seed count**: 10,000 todos

---

## 📝 Dependencies

```toml
[dependencies]
ratatui = "0.29"           # Terminal UI framework
crossterm = "0.28"         # Terminal manipulation
rusqlite = "0.32"          # SQLite bindings (bundled)
axum = "0.7"               # Web framework
tokio = "1"                # Async runtime
serde = "1.0"              # Serialization
tower-http = "0.5"         # CORS middleware
clap = "4.5"               # CLI argument parsing
```

---

## 🤝 Contributing

This is a demonstration project showcasing:
- Clean architecture in Rust
- Multiple interface patterns (TUI, CLI, REST API)
- Comprehensive testing strategies
- Pagination with large datasets
- Database design patterns

Feel free to fork and extend!

---

## 📄 License

MIT License - See LICENSE file for details

---

## 🙏 Acknowledgments

- **Ratatui** - Excellent TUI framework
- **Clap** - Fantastic CLI builder
- **Axum** - Modern, ergonomic web framework
- **SQLite** - Reliable embedded database

---

<div align="center">

**Built with ❤️ and 🦀 Rust**

⭐ Star this repo if you find it useful!

</div>
