use duckdb;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum TodoError {
    DuckDB(duckdb::Error),
    Io(io::Error),
    Custom(String),
}

impl fmt::Display for TodoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TodoError::DuckDB(err) => write!(f, "DuckDB error: {}", err),
            TodoError::Io(err) => write!(f, "IO error: {}", err),
            TodoError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for TodoError {} // Implement Error for TodoError

impl From<duckdb::Error> for TodoError {
    fn from(error: duckdb::Error) -> Self {
        TodoError::DuckDB(error)
    }
}

impl From<io::Error> for TodoError {
    fn from(error: io::Error) -> Self {
        TodoError::Io(error)
    }
}
