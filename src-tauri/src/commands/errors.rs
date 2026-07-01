use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum AppError {
    Database(String),
    NotFound(String),
    Validation(String),
    Io(String),
    Routing(String),
    AI(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AppError::Io(msg) => write!(f, "I/O error: {}", msg),
            AppError::Routing(msg) => write!(f, "Routing error: {}", msg),
            AppError::AI(msg) => write!(f, "AI error: {}", msg),
        }
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self { AppError::Validation(s) }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self { AppError::Validation(s.to_string()) }
}
