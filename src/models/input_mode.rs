/// Represents the current input mode of the application
#[derive(Debug, PartialEq, Eq)]
pub enum InputMode {
    /// Normal mode for navigation and actions
    Normal,
    /// Editing mode for adding new todos
    Editing,
}
