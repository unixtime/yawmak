use crate::error::TodoError;
use crate::task::Task;
use chrono::{Duration, NaiveDate};
use duckdb::params;
use duckdb::types::ValueRef;
use duckdb::Connection;

pub struct Database {
    conn: Connection,
}

impl Database {
    fn setup_extensions(conn: &Connection) {
        // Redirect DuckDB's output temporarily
        conn.execute_batch("
            SET log_query_path='';
            SET logging_level='error';
            SET enable_progress_bar=false;
            
            BEGIN TRANSACTION;
            -- Install extensions if they don't exist
            CREATE TEMP TABLE IF NOT EXISTS _extension_check AS SELECT 1;
            INSTALL excel;
            INSTALL parquet;
            
            -- Load extensions
            LOAD excel;
            LOAD parquet;
            COMMIT;
            
            DROP TABLE IF EXISTS _extension_check;
        ").ok();
    }

    pub fn new(path: &str) -> Result<Self, TodoError> {
        let conn = Connection::open(path).map_err(TodoError::from)?;
        
        // Setup extensions first with suppressed output
        Self::setup_extensions(&conn);

        // Create base tables in a single transaction
        conn.execute_batch(
            "BEGIN;
        CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY,
            task TEXT NOT NULL,
            done BOOLEAN NOT NULL DEFAULT 0,
            due_date DATE,
            completion_date DATE,
            priority INTEGER DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL
        );
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL
        );
        CREATE TABLE IF NOT EXISTS todo_categories (
            todo_id INTEGER,
            category_id INTEGER,
            FOREIGN KEY(todo_id) REFERENCES todos(id),
            FOREIGN KEY(category_id) REFERENCES categories(id)
        );
        CREATE TABLE IF NOT EXISTS todo_tags (
            todo_id INTEGER,
            tag_id INTEGER,
            FOREIGN KEY(todo_id) REFERENCES todos(id),
            FOREIGN KEY(tag_id) REFERENCES tags(id)
        );
        COMMIT;"
        ).map_err(TodoError::from)?;

        Ok(Database { conn })
    }
    pub fn add_task(&self, task: Task) -> Result<(), TodoError> {
        let sql = "INSERT INTO todos (task, due_date, priority) VALUES (?1, ?2, ?3) RETURNING id";
        let due_date_str = task.due_date.map(|d| d.format("%Y-%m-%d").to_string());
        let last_id: i32 = self
            .conn
            .query_row(
                sql,
                params![&task.name, due_date_str.as_deref(), &task.priority],
                |row| row.get(0),
            )
            .map_err(TodoError::from)?;

        if let Some(ref category) = task.category {
            let _ = self.add_category(category);  // Ignore if category already exists
            if let Ok(category_id) = self.get_category_id(category) {
                let _ = self.conn.execute(
                    "INSERT INTO todo_categories (todo_id, category_id) VALUES (?1, ?2)",
                    &[&last_id, &category_id],
                );
            }
        }

        for tag in &task.tags {
            let _ = self.add_tag(tag);  // Ignore if tag already exists
            if let Ok(tag_id) = self.get_tag_id(tag) {
                let _ = self.conn.execute(
                    "INSERT INTO todo_tags (todo_id, tag_id) VALUES (?1, ?2)",
                    &[&last_id, &tag_id],
                );
            }
        }

        Ok(())
    }

    pub fn get_tasks(&self, done_only: Option<bool>) -> Result<Vec<Task>, TodoError> {
        let query = match done_only {
            Some(true) => "SELECT t.id, t.task, t.done, t.due_date, t.completion_date, t.priority, 
                       c.name as category_name 
                       FROM todos t 
                       LEFT JOIN todo_categories tc ON t.id = tc.todo_id 
                       LEFT JOIN categories c ON tc.category_id = c.id 
                       WHERE t.done = 1",
            Some(false) => "SELECT t.id, t.task, t.done, t.due_date, t.completion_date, t.priority, 
                        c.name as category_name 
                        FROM todos t 
                        LEFT JOIN todo_categories tc ON t.id = tc.todo_id 
                        LEFT JOIN categories c ON tc.category_id = c.id 
                        WHERE t.done = 0",
            None => "SELECT t.id, t.task, t.done, t.due_date, t.completion_date, t.priority, 
                 c.name as category_name 
                 FROM todos t 
                 LEFT JOIN todo_categories tc ON t.id = tc.todo_id 
                 LEFT JOIN categories c ON tc.category_id = c.id",
        };

        let mut stmt = self.conn.prepare(query).map_err(TodoError::from)?;
        let rows = stmt
            .query_map([], |row| {
                let id: i32 = row.get(0)?;
                let task: String = row.get(1)?;
                let done: bool = row.get(2)?;
                let due_date = match row.get_ref(3)? {
                    ValueRef::Date32(ref date32) => Some(
                        NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
                            + Duration::days(*date32 as i64),
                    ),
                    _ => None,
                };
                let completion_date = match row.get_ref(4)? {
                    ValueRef::Date32(ref date32) => Some(
                        NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
                            + Duration::days(*date32 as i64),
                    ),
                    _ => None,
                };
                let priority: i32 = row.get(5)?;
                let category: Option<String> = row.get(6).ok();

                // Get tags in a single query
                let tags = self.get_task_tags(id).unwrap_or_else(|_| vec![]);

                Ok(Task {
                    id,
                    name: task,
                    category,
                    tags,
                    done,
                    due_date,
                    completion_date,
                    priority,
                })
            })
            .map_err(TodoError::from)?;

        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row.map_err(TodoError::from)?);
        }
        Ok(tasks)
    }

    pub fn get_task_tags(&self, task_id: i32) -> Result<Vec<String>, TodoError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.name 
         FROM tags t 
         JOIN todo_tags tt ON t.id = tt.tag_id 
         WHERE tt.todo_id = ?1"
        ).map_err(TodoError::from)?;

        let rows = stmt
            .query_map([task_id], |row| row.get::<_, String>(0))
            .map_err(TodoError::from)?;

        let mut tags = Vec::new();
        for row in rows {
            tags.push(row.map_err(TodoError::from)?);
        }
        Ok(tags)
    }

    pub fn mark_task_done(&self, id: i32) -> Result<(), TodoError> {
        let sql = "UPDATE todos SET done = 1, completion_date = CURRENT_DATE WHERE id = ?1";
        self.conn.execute(sql, &[&id]).map_err(TodoError::from)?;
        Ok(())
    }

    pub fn update_task(
        &self,
        id: i32,
        new_task: Option<String>,
        new_due_date: Option<String>,
        new_category: Option<String>,
        new_tags: Vec<String>,
        new_priority: Option<i32>,
        mark_undone: bool,
    ) -> Result<(), TodoError> {
        let mut updates = vec![];

        if let Some(task) = new_task {
            updates.push(format!("task = '{}'", task));
        }
        if let Some(due_date) = new_due_date {
            updates.push(format!("due_date = '{}'", due_date));
        }
        if let Some(priority) = new_priority {
            updates.push(format!("priority = {}", priority));
        }
        if mark_undone {
            updates.push("done = 0".to_string());
            updates.push("completion_date = NULL".to_string());
        }

        if !updates.is_empty() {
            let sql = format!("UPDATE todos SET {} WHERE id = ?1", updates.join(", "));
            self.conn.execute(&sql, &[&id]).map_err(TodoError::from)?;
        }

        if let Some(category) = new_category {
            let _ = self.add_category(&category);
            if let Ok(category_id) = self.get_category_id(&category) {
                let _ = self.conn
                    .execute("DELETE FROM todo_categories WHERE todo_id = ?1", &[&id]);
                let _ = self.conn
                    .execute(
                        "INSERT INTO todo_categories (todo_id, category_id) VALUES (?1, ?2)",
                        &[&id, &category_id],
                    );
            }
        }

        if !new_tags.is_empty() {
            let _ = self.conn
                .execute("DELETE FROM todo_tags WHERE todo_id = ?1", &[&id]);

            let tags_list: Vec<&str> = new_tags
                .iter()
                .flat_map(|t| t.split(',').map(|s| s.trim()))
                .collect();

            for tag in tags_list {
                let _ = self.add_tag(tag);
                if let Ok(tag_id) = self.get_tag_id(tag) {
                    let _ = self.conn
                        .execute(
                            "INSERT INTO todo_tags (todo_id, tag_id) VALUES (?1, ?2)",
                            &[&id, &tag_id],
                        );
                }
            }
        }

        Ok(())
    }

    pub fn get_category_id(&self, name: &str) -> Result<i32, TodoError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM categories WHERE name = ?1")
            .map_err(TodoError::from)?;
        let id = stmt
            .query_row([name], |row| row.get(0))
            .map_err(TodoError::from)?;
        Ok(id)
    }

    pub fn get_tag_id(&self, name: &str) -> Result<i32, TodoError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM tags WHERE name = ?1")
            .map_err(TodoError::from)?;
        let id = stmt
            .query_row([name], |row| row.get(0))
            .map_err(TodoError::from)?;
        Ok(id)
    }

    pub fn add_category(&self, name: &str) -> Result<(), TodoError> {
        let sql = "INSERT OR IGNORE INTO categories (name) VALUES (?1)";
        self.conn.execute(sql, &[name]).map_err(TodoError::from)?;
        Ok(())
    }

    pub fn delete_category(&self, name: &str) -> Result<(), TodoError> {
        let sql = "DELETE FROM categories WHERE name = ?1";
        self.conn.execute(sql, &[name]).map_err(TodoError::from)?;
        Ok(())
    }

    pub fn list_categories(&self) -> Result<Vec<String>, TodoError> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM categories")
            .map_err(TodoError::from)?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(TodoError::from)?;
        let mut categories = Vec::new();
        for row in rows {
            categories.push(row.map_err(TodoError::from)?);
        }
        Ok(categories)
    }

    pub fn add_tag(&self, name: &str) -> Result<(), TodoError> {
        let sql = "INSERT OR IGNORE INTO tags (name) VALUES (?1)";
        self.conn.execute(sql, &[name]).map_err(TodoError::from)?;
        Ok(())
    }

    pub fn delete_tag(&self, name: &str) -> Result<(), TodoError> {
        let sql = "DELETE FROM tags WHERE name = ?1";
        self.conn.execute(sql, &[name]).map_err(TodoError::from)?;
        Ok(())
    }

    pub fn list_tags(&self) -> Result<Vec<String>, TodoError> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM tags")
            .map_err(TodoError::from)?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(TodoError::from)?;
        let mut tags = Vec::new();
        for row in rows {
            tags.push(row.map_err(TodoError::from)?);
        }
        Ok(tags)
    }

    pub fn import_from_json(&self, file_path: &str, strategy: &str) -> Result<(), TodoError> {
        let command = match strategy {
            "skip" => format!("INSERT OR IGNORE INTO todos SELECT * FROM read_json_auto('{}')", file_path),
            "remove" => format!("COPY todos (task, done, due_date, completion_date, priority) FROM '{}' (FORMAT JSON)", file_path),
            "upsert" => format!("INSERT OR REPLACE INTO todos SELECT * FROM read_json_auto('{}')", file_path),
            _ => return Err(TodoError::Custom("Unsupported strategy".into())),
        };
        self.conn.execute(&command, []).map_err(TodoError::from)?;
        Ok(())
    }

    pub fn import_from_csv(&self, file_path: &str, strategy: &str) -> Result<(), TodoError> {
        let command = match strategy {
            "skip" => format!("INSERT OR IGNORE INTO todos SELECT * FROM read_csv_auto('{}')", file_path),
            "remove" => format!("COPY todos (task, done, due_date, completion_date, priority) FROM '{}' (FORMAT CSV)", file_path),
            "upsert" => format!("INSERT OR REPLACE INTO todos SELECT * FROM read_csv_auto('{}')", file_path),
            _ => return Err(TodoError::Custom("Unsupported strategy".into())),
        };
        self.conn.execute(&command, []).map_err(TodoError::from)?;
        Ok(())
    }

    pub fn export_to_json(&self, file_path: &str) -> Result<(), TodoError> {
        self.conn
            .execute(
                &format!("COPY (SELECT * FROM todos) TO '{}' (FORMAT JSON)", file_path),
                [],
            )
            .map_err(TodoError::from)?;
        Ok(())
    }

    pub fn export_to_csv(&self, file_path: &str) -> Result<(), TodoError> {
        self.conn
            .execute(
                &format!("COPY (SELECT * FROM todos) TO '{}' (FORMAT CSV, HEADER)", file_path),
                [],
            )
            .map_err(TodoError::from)?;
        Ok(())
    }
    pub fn import_from_parquet(&self, file_path: &str, strategy: &str) -> Result<(), TodoError> {
        let command = match strategy {
            "skip" => format!("INSERT OR IGNORE INTO todos SELECT * FROM read_parquet('{}')", file_path),
            "remove" => format!("COPY todos (task, done, due_date, completion_date, priority) FROM '{}' (FORMAT PARQUET)", file_path),
            "upsert" => format!("INSERT OR REPLACE INTO todos SELECT * FROM read_parquet('{}')", file_path),
            _ => return Err(TodoError::Custom("Unsupported strategy".into())),
        };
        self.conn.execute(&command, []).map_err(TodoError::from)?;
        Ok(())
    }

    pub fn export_to_parquet(&self, file_path: &str) -> Result<(), TodoError> {
        self.conn
            .execute(
                &format!("COPY (SELECT * FROM todos) TO '{}' (FORMAT PARQUET)", file_path),
                [],
            )
            .map_err(TodoError::from)?;
        Ok(())
    }
    pub fn import_from_excel(&self, file_path: &str, strategy: &str) -> Result<(), TodoError> {
        let command = match strategy {
            "skip" => format!(
                "INSERT OR IGNORE INTO todos 
             SELECT * FROM read_xlsx('{}')",
                file_path
            ),
            "remove" => format!(
                "INSERT INTO todos (task, done, due_date, completion_date, priority) 
             SELECT task, done, due_date, completion_date, priority 
             FROM read_xlsx('{}')",
                file_path
            ),
            "upsert" => format!(
                "INSERT OR REPLACE INTO todos 
             SELECT * FROM read_xlsx('{}')",
                file_path
            ),
            _ => return Err(TodoError::Custom("Unsupported strategy".into())),
        };
        self.conn.execute(&command, []).map_err(TodoError::from)?;
        Ok(())
    }
    pub fn export_to_excel(&self, file_path: &str) -> Result<(), TodoError> {
        self.conn
            .execute(
                &format!(
                    "COPY (
                    SELECT t.*, 
                           c.name as category,
                           (SELECT string_agg(tags.name, ',') 
                            FROM todo_tags 
                            JOIN tags ON tags.id = todo_tags.tag_id 
                            WHERE todo_tags.todo_id = t.id) as tags
                    FROM todos t
                    LEFT JOIN todo_categories tc ON t.id = tc.todo_id
                    LEFT JOIN categories c ON c.id = tc.category_id
                ) TO '{}' (FORMAT XLSX, HEADER true)",
                    file_path
                ),
                [],
            )
            .map_err(TodoError::from)?;
        Ok(())
    }
}