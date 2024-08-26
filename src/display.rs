use crate::task::Task;
use prettytable::{Cell, Row, Table};

pub struct Display;

impl Display {
    pub fn show_tasks(tasks: Vec<Task>, show_completion_date: bool) {
        let mut table = Table::new();

        // Add table headers
        let mut headers = vec![
            "ID", "Name", "Category", "Tags", "Due Date", "Done", "Priority",
        ];

        // Add "Completion Date" header only if show_completion_date is true
        if show_completion_date {
            headers.push("Completion Date");
        }

        table.add_row(Row::new(
            headers
                .iter()
                .map(|header| Cell::new(header))
                .collect::<Vec<Cell>>(),
        ));

        // Add task rows
        for task in tasks {
            let mut row = vec![
                Cell::new(&task.id.to_string()),
                Cell::new(&task.name),
                Cell::new(&task.category.clone().unwrap_or_default()),
                Cell::new(&task.tags.join(", ")),
                Cell::new(
                    &task
                        .due_date
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_default(),
                ),
                Cell::new(&task.done.to_string()),
                Cell::new(&task.priority.to_string()),
            ];

            // Add "Completion Date" cell only if show_completion_date is true
            if show_completion_date {
                row.push(Cell::new(
                    &task
                        .completion_date
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_default(),
                ));
            }

            table.add_row(Row::new(row));
        }

        table.printstd();
    }

    pub fn show_categories(categories: Vec<String>) {
        let mut table = Table::new();
        table.add_row(Row::new(vec![Cell::new("Category")]));
        for category in categories {
            table.add_row(Row::new(vec![Cell::new(&category)]));
        }
        table.printstd();
    }

    pub fn show_tags(tags: Vec<String>) {
        let mut table = Table::new();
        table.add_row(Row::new(vec![Cell::new("Tag")]));
        for tag in tags {
            table.add_row(Row::new(vec![Cell::new(&tag)]));
        }
        table.printstd();
    }
}
