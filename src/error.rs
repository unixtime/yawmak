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
            TodoError::DuckDB(err) => {
                // Customize the error message based on DuckDB error details
                if err.to_string().contains("GDAL Error") {
                    write!(f, "There was an issue with file access: {}. Please ensure the file exists and the path is correct.", err)
                } else {
                    write!(f, "A database error occurred: {}. Please ensure your database setup is correct.", err)
                }
            },
            TodoError::Io(err) => write!(f, "There was an input/output error: {}. Please check your file paths and permissions.", err),
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
