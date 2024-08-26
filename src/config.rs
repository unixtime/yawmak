use std::env;
use std::path::PathBuf;

pub struct Config {
    db_path: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        let home_dir = env::var("HOME").unwrap();
        let db_path = PathBuf::from(format!("{}/.yawmak/db", home_dir));
        Config { db_path }
    }

    pub fn get_db_path(&self) -> &PathBuf {
        &self.db_path
    }
}
