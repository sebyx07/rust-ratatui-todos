use crate::app::App;
use crate::models::InputMode;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Render the UI for the application
pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_title(f, chunks[0]);
    render_todo_list(f, app, chunks[1]);
    render_pagination_bar(f, app, chunks[2]);
    render_status_bar(f, app, chunks[3]);
}

/// Render the title bar
fn render_title(f: &mut Frame, area: ratatui::layout::Rect) {
    let title = Paragraph::new("Rust Todo List with Ratatui & SQLite")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

/// Render the todo list
fn render_todo_list(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .todos
        .iter()
        .map(|todo| {
            let status = if todo.completed { "[x]" } else { "[ ]" };
            let style = if todo.completed {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::raw(status),
                Span::raw(" "),
                Span::styled(&todo.title, style),
            ]))
        })
        .collect();

    let items_widget = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Todos"))
        .highlight_style(
            Style::default()
                .bg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items_widget, area, &mut app.list_state);
}

/// Render the pagination bar
fn render_pagination_bar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let pagination_text = app.pagination_info();
    let pagination = Paragraph::new(pagination_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Pagination"));
    f.render_widget(pagination, area);
}

/// Render the status/help bar
fn render_status_bar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw(" quit | "),
                Span::styled("i/a", Style::default().fg(Color::Yellow)),
                Span::raw(" add | "),
                Span::styled("space", Style::default().fg(Color::Yellow)),
                Span::raw(" toggle | "),
                Span::styled("d", Style::default().fg(Color::Yellow)),
                Span::raw(" delete | "),
                Span::styled("Shift+C", Style::default().fg(Color::Red)),
                Span::raw(" clear all | "),
                Span::styled("n", Style::default().fg(Color::Green)),
                Span::raw(" next | "),
                Span::styled("p", Style::default().fg(Color::Green)),
                Span::raw(" prev"),
            ],
            Style::default(),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("New todo: "),
                Span::styled(&app.input, Style::default().fg(Color::Yellow)),
                Span::raw(" | "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(" save | "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw(" cancel"),
            ],
            Style::default(),
        ),
    };

    let help = Paragraph::new(Line::from(msg))
        .style(style)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(help, area);
}
