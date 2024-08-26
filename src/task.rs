use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)] // Add Clone here
pub struct Task {
    pub id: i32,
    pub name: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub done: bool,
    pub due_date: Option<NaiveDate>,
    pub completion_date: Option<NaiveDate>,
    pub priority: i32, // New field for priority
}

impl Task {
    pub fn new(
        name: &str,
        category: String,
        due_date: Option<String>,
        tags: Vec<String>,
        priority: i32,
    ) -> Self {
        let due_date_parsed = due_date.map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").unwrap());
        Task {
            id: 0,
            name: name.to_string(),
            category: Some(category),
            tags,
            done: false,
            due_date: due_date_parsed,
            completion_date: None,
            priority, // Initialize priority
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_create_task() {
        let name = "Test Task";
        let category = "Work".to_string();
        let due_date = Some("2024-12-31".to_string());
        let tags = vec!["urgent".to_string(), "important".to_string()];
        let priority = 5;

        let task = Task::new(
            name,
            category.clone(),
            due_date.clone(),
            tags.clone(),
            priority,
        );

        assert_eq!(task.name, name);
        assert_eq!(task.category, Some(category));
        assert_eq!(task.tags, tags);
        assert_eq!(task.done, false);
        assert_eq!(
            task.due_date,
            Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
        );
        assert_eq!(task.priority, priority);
        assert!(task.completion_date.is_none());
    }

    #[test]
    fn test_task_with_no_due_date() {
        let name = "Test Task";
        let category = "Work".to_string();
        let due_date: Option<String> = None;
        let tags = vec![];
        let priority = 0;

        let task = Task::new(
            name,
            category.clone(),
            due_date.clone(),
            tags.clone(),
            priority,
        );

        assert_eq!(task.name, name);
        assert_eq!(task.category, Some(category));
        assert_eq!(task.tags, tags);
        assert_eq!(task.done, false);
        assert!(task.due_date.is_none());
        assert_eq!(task.priority, priority);
        assert!(task.completion_date.is_none());
    }
}
