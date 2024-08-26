use crate::error::TodoError;
use crate::task::Task;
use chrono::{Duration, NaiveDate};
use duckdb::params;
use duckdb::types::ValueRef;
use duckdb::{Connection, OptionalExt};

pub struct Database {
    conn: Connection,
}

impl Database {
    // Import and export
    pub fn import_from_json(&self, file_path: &str, strategy: &str) -> Result<(), TodoError> {
        let command = match strategy {
            "skip" => format!("INSERT OR IGNORE INTO todos SELECT * FROM read_json_auto('{}')", file_path),
            "remove" => format!("COPY todos (task, done, due_date, completion_date, priority) FROM '{}' (FORMAT 'json')", file_path),
            "upsert" => format!("INSERT OR REPLACE INTO todos SELECT * FROM read_json_auto('{}')", file_path),
            _ => return Err(TodoError::Custom("Unsupported strategy".into())),
        };
        self.conn.execute(&command, []).map_err(TodoError::from)?;
        Ok(())
    }

    pub fn import_from_parquet(&self, file_path: &str, strategy: &str) -> Result<(), TodoError> {
        let command = match strategy {
            "skip" => format!("INSERT OR IGNORE INTO todos SELECT * FROM read_parquet('{}')", file_path),
            "remove" => format!("COPY todos (task, done, due_date, completion_date, priority) FROM '{}' (FORMAT 'parquet')", file_path),
            "upsert" => format!("INSERT OR REPLACE INTO todos SELECT * FROM read_parquet('{}')", file_path),
            _ => return Err(TodoError::Custom("Unsupported strategy".into())),
        };
        self.conn.execute(&command, []).map_err(TodoError::from)?;
        Ok(())
    }

    pub fn import_from_excel(&self, file_path: &str, strategy: &str) -> Result<(), TodoError> {
        self.conn
            .execute("INSTALL spatial;", [])
            .map_err(TodoError::from)?;
        self.conn
            .execute("LOAD spatial;", [])
            .map_err(TodoError::from)?;

        let sheet_name = file_path.strip_suffix(".xlsx").unwrap_or(file_path);

        let command = match strategy {
            "skip" => format!("INSERT OR IGNORE INTO todos SELECT * FROM st_read('{}', layer='{}')", file_path, sheet_name),
            "remove" => format!("INSERT INTO todos (task, done, due_date, completion_date, priority) SELECT task, done, due_date, completion_date, priority FROM st_read('{}', layer='{}')", file_path, sheet_name),
            "upsert" => format!("INSERT OR REPLACE INTO todos SELECT * FROM st_read('{}', layer='{}')", file_path, sheet_name),
            _ => return Err(TodoError::Custom("Unsupported strategy".into())),
        };
        self.conn.execute(&command, []).map_err(TodoError::from)?;
        Ok(())
    }

    pub fn import_from_csv(&self, file_path: &str, strategy: &str) -> Result<(), TodoError> {
        let command = match strategy {
            "skip" => format!("INSERT OR IGNORE INTO todos SELECT * FROM read_csv_auto('{}')", file_path),
            "remove" => format!("COPY todos (task, done, due_date, completion_date, priority) FROM '{}' (FORMAT 'csv')", file_path),
            "upsert" => format!("INSERT OR REPLACE INTO todos SELECT * FROM read_csv_auto('{}')", file_path),
            _ => return Err(TodoError::Custom("Unsupported strategy".into())),
        };
        self.conn.execute(&command, []).map_err(TodoError::from)?;
        Ok(())
    }

    pub fn export_to_json(&self, file_path: &str) -> Result<(), TodoError> {
        self.conn
            .execute(
                &format!("COPY todos TO '{}' (FORMAT 'json')", file_path),
                [],
            )
            .map_err(TodoError::from)?;
        Ok(())
    }

    pub fn export_to_parquet(&self, file_path: &str) -> Result<(), TodoError> {
        self.conn
            .execute(
                &format!("COPY todos TO '{}' (FORMAT 'parquet')", file_path),
                [],
            )
            .map_err(TodoError::from)?;
        Ok(())
    }

    pub fn export_to_excel(&self, file_path: &str) -> Result<(), TodoError> {
        self.conn
            .execute(
                &format!(
                    "COPY (SELECT * FROM todos) TO '{}' WITH (FORMAT GDAL, DRIVER 'xlsx')",
                    file_path
                ),
                [],
            )
            .map_err(TodoError::from)?;
        Ok(())
    }

    pub fn export_to_csv(&self, file_path: &str) -> Result<(), TodoError> {
        self.conn
            .execute(&format!("COPY todos TO '{}' (FORMAT 'csv')", file_path), [])
            .map_err(TodoError::from)?;
        Ok(())
    }

    pub fn new(path: &str) -> Result<Self, TodoError> {
        let conn = Connection::open(path).map_err(TodoError::from)?;

        // Install and load the required extensions
        conn.execute("INSTALL 'excel';", [])
            .map_err(TodoError::from)?;
        conn.execute("LOAD 'excel';", []).map_err(TodoError::from)?;

        // Install and load spatial extension for additional functions
        conn.execute("INSTALL 'spatial';", [])
            .map_err(TodoError::from)?;
        conn.execute("LOAD 'spatial';", [])
            .map_err(TodoError::from)?;

        // Additional setup and table creation code...
        conn.execute("CREATE SEQUENCE IF NOT EXISTS todo_id_seq", [])
            .map_err(TodoError::from)?;
        conn.execute("CREATE SEQUENCE IF NOT EXISTS category_id_seq", [])
            .map_err(TodoError::from)?;
        conn.execute("CREATE SEQUENCE IF NOT EXISTS tag_id_seq", [])
            .map_err(TodoError::from)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS todos (
                id INTEGER DEFAULT nextval('todo_id_seq') PRIMARY KEY,
                task TEXT NOT NULL,
                done BOOLEAN NOT NULL DEFAULT 0,
                due_date DATE,
                completion_date DATE,
                priority INTEGER DEFAULT 0
            )",
            [],
        )
        .map_err(TodoError::from)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
                id INTEGER DEFAULT nextval('category_id_seq') PRIMARY KEY,
                name TEXT UNIQUE NOT NULL
            )",
            [],
        )
        .map_err(TodoError::from)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER DEFAULT nextval('tag_id_seq') PRIMARY KEY,
                name TEXT UNIQUE NOT NULL
            )",
            [],
        )
        .map_err(TodoError::from)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS todo_categories (
                todo_id INTEGER,
                category_id INTEGER,
                FOREIGN KEY(todo_id) REFERENCES todos(id),
                FOREIGN KEY(category_id) REFERENCES categories(id)
            )",
            [],
        )
        .map_err(TodoError::from)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS todo_tags (
                todo_id INTEGER,
                tag_id INTEGER,
                FOREIGN KEY(todo_id) REFERENCES todos(id),
                FOREIGN KEY(tag_id) REFERENCES tags(id)
            )",
            [],
        )
        .map_err(TodoError::from)?;

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
            self.add_category(category)?;
            let category_id = self.get_category_id(category)?;
            self.conn
                .execute(
                    "INSERT INTO todo_categories (todo_id, category_id) VALUES (?1, ?2)",
                    &[&last_id, &category_id],
                )
                .map_err(TodoError::from)?;
        }

        for tag in &task.tags {
            self.add_tag(tag)?;
            let tag_id = self.get_tag_id(tag)?;
            self.conn
                .execute(
                    "INSERT INTO todo_tags (todo_id, tag_id) VALUES (?1, ?2)",
                    &[&last_id, &tag_id],
                )
                .map_err(TodoError::from)?;
        }

        Ok(())
    }

    pub fn get_tasks(&self, done_only: Option<bool>) -> Result<Vec<Task>, TodoError> {
        let query = match done_only {
            Some(true) => {
                "SELECT id, task, done, due_date, completion_date, priority FROM todos WHERE done = 1"
            }
            Some(false) => {
                "SELECT id, task, done, due_date, completion_date, priority FROM todos WHERE done = 0"
            }
            None => "SELECT id, task, done, due_date, completion_date, priority FROM todos",
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

                // Handle errors properly by mapping them to TodoError
                let category = self.get_task_category(id).unwrap_or_else(|_| None);
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

    pub fn get_task_category(&self, task_id: i32) -> Result<Option<String>, TodoError> {
        let mut stmt = self.conn.prepare(
            "SELECT c.name FROM categories c JOIN todo_categories tc ON c.id = tc.category_id WHERE tc.todo_id = ?1",
        ).map_err(TodoError::from)?;
        let category = stmt
            .query_row([task_id], |row| row.get(0))
            .optional()
            .map_err(TodoError::from)?;
        Ok(category)
    }

    pub fn get_task_tags(&self, task_id: i32) -> Result<Vec<String>, TodoError> {
        let mut stmt = self.conn.prepare(
            "SELECT t.name FROM tags t JOIN todo_tags tt ON t.id = tt.tag_id WHERE tt.todo_id = ?1",
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
            self.add_category(&category)?;
            let category_id = self.get_category_id(&category)?;
            self.conn
                .execute("DELETE FROM todo_categories WHERE todo_id = ?1", &[&id])
                .map_err(TodoError::from)?;
            self.conn
                .execute(
                    "INSERT INTO todo_categories (todo_id, category_id) VALUES (?1, ?2)",
                    &[&id, &category_id],
                )
                .map_err(TodoError::from)?;
        }

        if !new_tags.is_empty() {
            self.conn
                .execute("DELETE FROM todo_tags WHERE todo_id = ?1", &[&id])
                .map_err(TodoError::from)?;

            // Split tags by comma and trim them
            let tags_list: Vec<&str> = new_tags
                .iter()
                .flat_map(|t| t.split(',').map(|s| s.trim()))
                .collect();

            for tag in tags_list {
                self.add_tag(tag)?;
                let tag_id = self.get_tag_id(tag)?;
                self.conn
                    .execute(
                        "INSERT INTO todo_tags (todo_id, tag_id) VALUES (?1, ?2)",
                        &[&id, &tag_id],
                    )
                    .map_err(TodoError::from)?;
            }
        }

        Ok(())
    }

    fn get_category_id(&self, name: &str) -> Result<i32, TodoError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM categories WHERE name = ?1")
            .map_err(TodoError::from)?;
        let id = stmt
            .query_row([name], |row| row.get(0))
            .map_err(TodoError::from)?;
        Ok(id)
    }

    fn get_tag_id(&self, name: &str) -> Result<i32, TodoError> {
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

        // Check if the category was actually added
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM categories WHERE name = ?1")
            .map_err(TodoError::from)?;
        let count: i32 = stmt
            .query_row([name], |row| row.get(0))
            .map_err(TodoError::from)?;

        if count == 0 {
            return Err(TodoError::Custom("Category already exists.".into()));
        }

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

        // Check if the tag was actually added
        let mut stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM tags WHERE name = ?1")
            .map_err(TodoError::from)?;
        let count: i32 = stmt
            .query_row([name], |row| row.get(0))
            .map_err(TodoError::from)?;

        if count == 0 {
            return Err(TodoError::Custom("Tag already exists.".into()));
        }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;
    use chrono::NaiveDate;

    fn setup_test_db() -> Database {
        let db = Database::new(":memory:").unwrap();

        // Install and load the necessary extensions for testing
        db.conn.execute("INSTALL 'excel';", []).unwrap();
        db.conn.execute("LOAD 'excel';", []).unwrap();
        db.conn.execute("INSTALL 'spatial';", []).unwrap();
        db.conn.execute("LOAD 'spatial';", []).unwrap();

        db
    }

    #[test]
    fn test_add_and_get_task() {
        let db = setup_test_db();

        let task = Task::new(
            "Test Task",
            "Personal".to_string(),
            Some("2024-12-31".to_string()),
            vec!["test".to_string()],
            1,
        );

        // Add task to the database
        db.add_task(task.clone()).unwrap();

        // Retrieve tasks from the database
        let tasks = db.get_tasks(None).unwrap();
        assert_eq!(tasks.len(), 1);

        let db_task = &tasks[0];
        assert_eq!(db_task.name, task.name);
        assert_eq!(db_task.category, task.category);
        assert_eq!(db_task.due_date, task.due_date);
        assert_eq!(db_task.tags, task.tags);
        assert_eq!(db_task.priority, task.priority);
    }

    #[test]
    fn test_mark_task_done() {
        let db = setup_test_db();

        let task = Task::new(
            "Test Task",
            "Personal".to_string(),
            Some("2024-12-31".to_string()),
            vec![],
            1,
        );

        db.add_task(task).unwrap();
        let tasks = db.get_tasks(None).unwrap();
        let task_id = tasks[0].id;

        // Mark task as done
        db.mark_task_done(task_id).unwrap();

        // Verify task is marked as done
        let tasks = db.get_tasks(Some(true)).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].done, true);
    }

    #[test]
    fn test_update_task() {
        let db = setup_test_db();

        let task = Task::new(
            "Test Task",
            "Personal".to_string(),
            Some("2024-12-31".to_string()),
            vec!["test".to_string()],
            1,
        );

        db.add_task(task.clone()).unwrap();
        let tasks = db.get_tasks(None).unwrap();
        let task_id = tasks[0].id;

        // Update task details
        db.update_task(
            task_id,
            Some("Updated Task".to_string()),
            Some("2025-01-01".to_string()),
            Some("Work".to_string()),
            vec!["updated".to_string()],
            Some(2),
            false,
        )
        .unwrap();

        // Retrieve updated task
        let updated_task = db.get_tasks(None).unwrap()[0].clone();
        assert_eq!(updated_task.name, "Updated Task");
        assert_eq!(updated_task.category, Some("Work".to_string()));
        assert_eq!(
            updated_task.due_date,
            Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
        );
        assert_eq!(updated_task.tags, vec!["updated".to_string()]);
        assert_eq!(updated_task.priority, 2);
    }
}
