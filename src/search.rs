use crate::database::Database;
use crate::task::Task;

pub struct Search;

impl Search {
    pub fn find_tasks(db: &Database, query: &str) -> Vec<Task> {
        db.get_tasks(None)
            .unwrap_or_default()
            .into_iter()
            .filter(|t| {
                t.name.contains(query)
                    || t.category.as_deref().map_or(false, |c| c.contains(query))  // Correct usage
                    || t.tags.iter().any(|tag| tag.contains(query))
            })
            .collect()
    }
}
