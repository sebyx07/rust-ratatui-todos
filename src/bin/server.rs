use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use rust_ratatui_todo::{db::Database, models::Todo};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::cors::CorsLayer;

/// Application state shared across handlers
struct AppState {
    db: Arc<Mutex<Database>>,
}

/// Request body for creating a new todo
#[derive(Debug, Deserialize)]
struct CreateTodoRequest {
    title: String,
}

/// Request body for updating a todo
#[derive(Debug, Deserialize)]
struct UpdateTodoRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    completed: Option<bool>,
}

/// Query parameters for paginated list
#[derive(Debug, Deserialize)]
struct PaginationParams {
    #[serde(default)]
    page: Option<u32>,
    #[serde(default = "default_page_size")]
    page_size: u32,
}

fn default_page_size() -> u32 {
    20
}

/// Response for paginated list of todos
#[derive(Debug, Serialize)]
struct PaginatedTodosResponse {
    todos: Vec<Todo>,
    page: u32,
    page_size: u32,
    total_count: u32,
    total_pages: u32,
}

/// Standard error response
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

/// Custom error type for API handlers
enum ApiError {
    DatabaseError(rusqlite::Error),
    NotFound,
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::DatabaseError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", err),
            ),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Todo not found".to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

impl From<rusqlite::Error> for ApiError {
    fn from(err: rusqlite::Error) -> Self {
        ApiError::DatabaseError(err)
    }
}

/// GET /todos - List all todos with optional pagination
async fn list_todos(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedTodosResponse>, ApiError> {
    let db = state.db.lock().unwrap();

    let page = params.page.unwrap_or(0);
    let page_size = params.page_size;

    let total_count = db.count_todos()?;
    let todos = db.get_todos_paginated(page, page_size)?;

    let total_pages = if total_count == 0 {
        1
    } else {
        total_count.div_ceil(page_size)
    };

    Ok(Json(PaginatedTodosResponse {
        todos,
        page,
        page_size,
        total_count,
        total_pages,
    }))
}

/// GET /todos/:id - Get a single todo by ID
async fn get_todo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Todo>, ApiError> {
    let db = state.db.lock().unwrap();
    let todo = db.get_todo_by_id(id)?;
    todo.map(Json).ok_or(ApiError::NotFound)
}

/// POST /todos - Create a new todo
async fn create_todo(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTodoRequest>,
) -> Result<(StatusCode, Json<Todo>), ApiError> {
    if payload.title.trim().is_empty() {
        return Err(ApiError::BadRequest("Title cannot be empty".to_string()));
    }

    let db = state.db.lock().unwrap();
    db.add_todo(&payload.title)?;

    // Get the newly created todo (last one in the list)
    let todos = db.get_todos()?;
    let new_todo = todos
        .last()
        .ok_or_else(|| ApiError::DatabaseError(rusqlite::Error::QueryReturnedNoRows))?
        .clone();

    Ok((StatusCode::CREATED, Json(new_todo)))
}

/// PUT /todos/:id - Update a todo
async fn update_todo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTodoRequest>,
) -> Result<Json<Todo>, ApiError> {
    let db = state.db.lock().unwrap();

    // Verify todo exists
    let todo = db.get_todo_by_id(id)?.ok_or(ApiError::NotFound)?;

    // Update title if provided
    if let Some(title) = &payload.title {
        if title.trim().is_empty() {
            return Err(ApiError::BadRequest("Title cannot be empty".to_string()));
        }
        db.update_todo_title(id, title)?;
    }

    // Toggle completion if the value changed
    if let Some(completed) = payload.completed
        && completed != todo.completed
    {
        db.toggle_todo(id)?;
    }

    // Return the updated todo
    let updated_todo = db.get_todo_by_id(id)?.ok_or(ApiError::NotFound)?;
    Ok(Json(updated_todo))
}

/// DELETE /todos/:id - Delete a todo
async fn delete_todo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let db = state.db.lock().unwrap();
    db.delete_todo(id)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let db_path = std::env::var("TODO_DB_PATH").unwrap_or_else(|_| "./tmp/todos.db".to_string());

    // Ensure the directory exists
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db = Database::new(&db_path)?;
    println!("Database initialized at: {}", db_path);

    let state = Arc::new(AppState {
        db: Arc::new(Mutex::new(db)),
    });

    // Build the router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/todos", get(list_todos).post(create_todo))
        .route(
            "/todos/:id",
            get(get_todo).put(update_todo).delete(delete_todo),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Todo server listening on http://{}", addr);
    println!("\nAvailable endpoints:");
    println!("  GET    /health       - Health check");
    println!("  GET    /todos        - List todos (supports ?page=0&page_size=20)");
    println!("  GET    /todos/:id    - Get a specific todo");
    println!("  POST   /todos        - Create a new todo");
    println!("  PUT    /todos/:id    - Update a todo");
    println!("  DELETE /todos/:id    - Delete a todo");

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
