/// Represents a todo item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

impl Todo {
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn new(id: i64, title: impl Into<String>, completed: bool) -> Self {
        Self {
            id,
            title: title.into(),
            completed,
        }
    }
}
