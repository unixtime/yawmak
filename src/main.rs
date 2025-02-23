mod config;
mod database;
mod display;
mod error;
mod search;
mod task;

use crate::config::Config;
use crate::database::Database;
use crate::display::Display;
use crate::error::TodoError;
use crate::search::Search;
use crate::task::Task;
use chrono::NaiveDate;
use clap::{Arg, Command};
use clap_complete::{
    generate,
    shells::{Bash, Fish, PowerShell, Zsh},
};
use std::fs;
use std::io;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Oops! Something went wrong: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), TodoError> {
    let config = Config::new();
    let db_path = config.get_db_path();

    if let Some(db_dir) = db_path.parent() {
        if !db_dir.exists() {
            fs::create_dir_all(db_dir)?;
        }
    }

    let conn = Database::new(db_path.to_str().unwrap())?;

    let mut cmd = build_cli();
    let matches = cmd.clone().get_matches();

    match matches.subcommand() {
        Some(("completion", sub_m)) => {
            handle_completion(&mut cmd, sub_m);
        }
        Some(("add", sub_m)) => {
            handle_add(&conn, sub_m);
        }
        Some(("list", sub_m)) => {
            handle_list(&conn, sub_m)?;
        }
        Some(("done", sub_m)) => {
            handle_done(&conn, sub_m);
        }
        Some(("update", sub_m)) => {
            handle_update(&conn, sub_m);
        }
        Some(("search", sub_m)) => {
            handle_search(&conn, sub_m);
        }
        Some(("add-category", sub_m)) => {
            handle_add_category(&conn, sub_m);
        }
        Some(("delete-category", sub_m)) => {
            handle_delete_category(&conn, sub_m);
        }
        Some(("list-categories", _)) => {
            handle_list_categories(&conn)?;
        }
        Some(("add-tag", sub_m)) => {
            handle_add_tag(&conn, sub_m);
        }
        Some(("delete-tag", sub_m)) => {
            handle_delete_tag(&conn, sub_m);
        }
        Some(("list-tags", _)) => {
            handle_list_tags(&conn)?;
        }
        Some(("import", sub_m)) => {
            handle_import(&conn, sub_m)?;
        }
        Some(("export", sub_m)) => {
            handle_export(&conn, sub_m)?;
        }
        _ => {
            println!("Invalid command. Use --help for available commands.");
        }
    }

    Ok(())
}

fn build_cli() -> Command {
    Command::new("yawmak")
        .version("1.1.2")
        .author("Hassan El-Masri <hassan@unixtime.com>")
        .about("Manages your todos")
        .subcommand(
            Command::new("add")
                .about(
                    "Adds a new todo task with an optional due date, category, tags, and priority.",
                )
                .arg(
                    Arg::new("TASK")
                        .help("The task description.")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("DUE_DATE")
                        .help("The due date for the task in YYYY-MM-DD format.")
                        .required(false)
                        .index(2),
                )
                .arg(
                    Arg::new("category")
                        .long("category")
                        .help("The category of the task.")
                        .value_name("CATEGORY")
                        .required(false),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .help("Tags associated with the task.")
                        .value_name("TAGS")
                        .num_args(1..)
                        .required(false),
                )
                .arg(
                    Arg::new("priority")
                        .long("priority")
                        .help("Priority of the task.")
                        .value_name("PRIORITY")
                        .required(false)
                        .default_value("0"),
                ),
        )
        .subcommand(
            Command::new("list")
                .about("Lists all todos, optionally filtering by done status.")
                .arg(
                    Arg::new("done-only")
                        .long("done-only")
                        .help("Lists only completed tasks.")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("done")
                .about("Marks a todo task as done.")
                .arg(
                    Arg::new("ID")
                        .help("The ID of the todo task.")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("update")
                .about("Updates an existing todo task's details.")
                .arg(
                    Arg::new("ID")
                        .help("The ID of the todo task to update.")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("TASK")
                        .long("task")
                        .help("The new task description.")
                        .value_name("TASK")
                        .required(false),
                )
                .arg(
                    Arg::new("DUE_DATE")
                        .long("due-date")
                        .help("The new due date for the task in YYYY-MM-DD format.")
                        .value_name("DUE_DATE")
                        .required(false),
                )
                .arg(
                    Arg::new("category")
                        .long("category")
                        .help("The new category of the task.")
                        .value_name("CATEGORY")
                        .required(false),
                )
                .arg(
                    Arg::new("tags")
                        .long("tags")
                        .help("New tags associated with the task.")
                        .value_name("TAGS")
                        .num_args(1..)
                        .required(false),
                )
                .arg(
                    Arg::new("priority")
                        .long("priority")
                        .help("The new priority of the task.")
                        .value_name("PRIORITY")
                        .required(false),
                )
                .arg(
                    Arg::new("undone")
                        .long("undone")
                        .help("Marks the task as not done.")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("search")
                .about("Searches tasks by name, due date, category, or tags.")
                .arg(Arg::new("QUERY").help("The search query.").required(true)),
        )
        .subcommand(
            Command::new("add-category")
                .about("Adds a new category.")
                .arg(
                    Arg::new("CATEGORY_NAME")
                        .help("The name of the category.")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("delete-category")
                .about("Deletes a category.")
                .arg(
                    Arg::new("CATEGORY_NAME")
                        .help("The name of the category to delete.")
                        .required(true),
                ),
        )
        .subcommand(Command::new("list-categories").about("Lists all categories."))
        .subcommand(
            Command::new("add-tag").about("Adds a new tag.").arg(
                Arg::new("TAG_NAME")
                    .help("The name of the tag.")
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("delete-tag").about("Deletes a tag.").arg(
                Arg::new("TAG_NAME")
                    .help("The name of the tag to delete.")
                    .required(true),
            ),
        )
        .subcommand(Command::new("list-tags").about("Lists all tags."))
        .subcommand(
            Command::new("completion")
                .about("Generate shell completion scripts for your shell")
                .arg(
                    Arg::new("shell")
                        .help("The shell to generate the completion script for")
                        .required(true)
                        .value_parser(["bash", "zsh", "fish", "powershell"]),
                ),
        )
        .subcommand(
            Command::new("import")
                .about("Import data into the todo list from a file")
                .arg(
                    Arg::new("format")
                        .help("The format of the file (json, parquet, xlsx, csv)")
                        .required(true),
                )
                .arg(
                    Arg::new("file")
                        .help("The file path to import from")
                        .required(true),
                )
                .arg(
                    Arg::new("strategy")
                        .help("The import strategy (skip, remove, upsert)")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("export")
                .about("Export data from the todo list to a file")
                .arg(
                    Arg::new("format")
                        .help("The format of the file (json, parquet, xlsx, csv)")
                        .required(true),
                )
                .arg(
                    Arg::new("file")
                        .help("The file path to export to")
                        .required(true),
                ),
        )
}

fn handle_completion(cmd: &mut Command, sub_m: &clap::ArgMatches) {
    let shell = sub_m.get_one::<String>("shell").unwrap();
    match shell.as_str() {
        "bash" => generate(Bash, cmd, "yawmak", &mut io::stdout()),
        "zsh" => generate(Zsh, cmd, "yawmak", &mut io::stdout()),
        "fish" => generate(Fish, cmd, "yawmak", &mut io::stdout()),
        "powershell" => generate(PowerShell, cmd, "yawmak", &mut io::stdout()),
        _ => println!("Unsupported shell"),
    }
}

fn handle_add(conn: &Database, sub_m: &clap::ArgMatches) {
    let task_description = sub_m.get_one::<String>("TASK").unwrap();
    let due_date = sub_m.get_one::<String>("DUE_DATE").map(|d| d.to_string());
    let category = sub_m
        .get_one::<String>("category")
        .unwrap_or(&"General".to_string())
        .to_string();

    // Correctly split the tags by comma
    let tags: Vec<String> = sub_m
        .get_many::<String>("tags")
        .unwrap_or_default()
        .flat_map(|v| v.split(',').map(|s| s.trim().to_string()))
        .collect();

    let priority: i32 = sub_m
        .get_one::<String>("priority")
        .unwrap()
        .parse()
        .unwrap_or_else(|_| {
            eprintln!("Invalid priority value. Please enter a valid integer.");
            process::exit(1);
        });

    let task = Task::new(task_description, category, due_date, tags, priority);
    if let Err(e) = conn.add_task(task) {
        handle_db_error(e);
    }
}

fn handle_list(conn: &Database, sub_m: &clap::ArgMatches) -> Result<(), TodoError> {
    let done_only = *sub_m.get_one::<bool>("done-only").unwrap_or(&false);
    let tasks = conn.get_tasks(Some(done_only))?;
    Display::show_tasks(tasks, done_only);
    Ok(())
}

fn handle_done(conn: &Database, sub_m: &clap::ArgMatches) {
    let id = parse_id(sub_m);
    if let Err(e) = conn.mark_task_done(id) {
        handle_db_error(e);
    }
}

// Common function to handle updating tasks
fn parse_id(sub_m: &clap::ArgMatches) -> i32 {
    sub_m
        .get_one::<String>("ID")
        .unwrap()
        .parse::<i32>()
        .unwrap_or_else(|_| {
            eprintln!("The ID you entered doesn't seem to be valid. Please enter a number, like 1 or 2, and try again.");
            process::exit(1);
        })
}

fn parse_due_date(due_date: Option<&String>) -> Option<String> {
    due_date.map(|d| {
        if NaiveDate::parse_from_str(d, "%Y-%m-%d").is_err() {
            eprintln!("Invalid date format. Please use YYYY-MM-DD.");
            process::exit(1);
        }
        d.to_string()
    })
}


fn handle_update(conn: &Database, sub_m: &clap::ArgMatches) {
    let id = parse_id(sub_m);
    let new_task = sub_m.get_one::<String>("TASK").map(|d| d.to_string());
    let new_due_date = parse_due_date(sub_m.get_one::<String>("DUE_DATE"));
    let new_category = sub_m.get_one::<String>("category").map(|d| d.to_string());
    let new_tags: Vec<String> = sub_m
        .get_many::<String>("tags")
        .unwrap_or_default()
        .map(|v| v.to_string())
        .collect();
    let new_priority = sub_m.get_one::<String>("priority").map(|p| {
        p.parse::<i32>().unwrap_or_else(|_| {
            eprintln!("Invalid priority value. Please enter a valid integer.");
            process::exit(1);
        })
    });
    let mark_undone = *sub_m.get_one::<bool>("undone").unwrap_or(&false);

    if let Err(e) = conn.update_task(
        id,
        new_task,
        new_due_date,
        new_category,
        new_tags,
        new_priority,
        mark_undone,
    ) {
        handle_db_error(e);
    }
}

fn handle_search(conn: &Database, sub_m: &clap::ArgMatches) {
    let query = sub_m.get_one::<String>("QUERY").unwrap();
    let results = Search::find_tasks(conn, query);
    Display::show_tasks(results, true);
}

fn handle_add_category(conn: &Database, sub_m: &clap::ArgMatches) {
    let category_name = sub_m.get_one::<String>("CATEGORY_NAME").unwrap();
    if let Err(e) = conn.add_category(category_name) {
        if e.to_string().to_lowercase().contains("constraint") {
            println!("Error: A category with the same name already exists.");
        } else {
            println!("An error occurred while adding the category: {}", e);
        }
    } else {
        println!("Added category: {}", category_name);
    }
}

fn handle_delete_category(conn: &Database, sub_m: &clap::ArgMatches) {
    let category_name = sub_m.get_one::<String>("CATEGORY_NAME").unwrap();
    if let Err(e) = conn.delete_category(category_name) {
        if e.to_string().to_lowercase().contains("foreign key") {
            println!("Error: Cannot delete category because it is still used by some tasks.");
        } else {
            println!("An error occurred while deleting the category: {}", e);
        }
    } else {
        println!("Deleted category: {}", category_name);
    }
}

fn handle_list_categories(conn: &Database) -> Result<(), TodoError> {
    let categories = conn.list_categories()?;
    Display::show_categories(categories);
    Ok(())
}

fn handle_add_tag(conn: &Database, sub_m: &clap::ArgMatches) {
    let tag_name = sub_m.get_one::<String>("TAG_NAME").unwrap();
    if let Err(e) = conn.add_tag(tag_name) {
        if e.to_string().to_lowercase().contains("constraint") {
            println!("Error: A tag with the same name already exists.");
        } else {
            println!("An error occurred while adding the tag: {}", e);
        }
    } else {
        println!("Added tag: {}", tag_name);
    }
}

fn handle_delete_tag(conn: &Database, sub_m: &clap::ArgMatches) {
    let tag_name = sub_m.get_one::<String>("TAG_NAME").unwrap();
    if let Err(e) = conn.delete_tag(tag_name) {
        if e.to_string().to_lowercase().contains("foreign key") {
            println!("Error: Cannot delete tag because it is still used by some tasks.");
        } else {
            println!("An error occurred while deleting the tag: {}", e);
        }
    } else {
        println!("Deleted tag: {}", tag_name);
    }
}

fn handle_list_tags(conn: &Database) -> Result<(), TodoError> {
    let tags = conn.list_tags()?;
    Display::show_tags(tags);
    Ok(())
}

fn handle_import(conn: &Database, sub_m: &clap::ArgMatches) -> Result<(), TodoError> {
    let format = sub_m.get_one::<String>("format").unwrap();
    let file_path = sub_m.get_one::<String>("file").unwrap();
    let strategy = sub_m.get_one::<String>("strategy").unwrap();

    match format.as_str() {
        "json" => {
            conn.import_from_json(file_path, strategy)?;
            println!(
                "Data imported successfully from JSON with strategy '{}'.",
                strategy
            );
        }
        "parquet" => {
            conn.import_from_parquet(file_path, strategy)?;
            println!(
                "Data imported successfully from Parquet with strategy '{}'.",
                strategy
            );
        }
        "xlsx" => {
            conn.import_from_excel(file_path, strategy)?;
            println!(
                "Data imported successfully from Excel with strategy '{}'.",
                strategy
            );
        }
        "csv" => {
            conn.import_from_csv(file_path, strategy)?;
            println!(
                "Data imported successfully from CSV with strategy '{}'.",
                strategy
            );
        }
        _ => {
            println!("Unsupported format. Please use json, parquet, xlsx, or csv.");
        }
    }

    Ok(())
}

fn handle_export(conn: &Database, sub_m: &clap::ArgMatches) -> Result<(), TodoError> {
    let format = sub_m.get_one::<String>("format").unwrap();
    let file_path = sub_m.get_one::<String>("file").unwrap();

    match format.as_str() {
        "json" => {
            conn.export_to_json(file_path)?;
            println!("Data exported successfully to JSON.");
        }
        "parquet" => {
            conn.export_to_parquet(file_path)?;
            println!("Data exported successfully to Parquet.");
        }
        "xlsx" => {
            conn.export_to_excel(file_path)?;
            println!("Data exported successfully to Excel.");
        }
        "csv" => {
            conn.export_to_csv(file_path)?;
            println!("Data exported successfully to CSV.");
        }
        _ => {
            println!("Unsupported format. Please use json, parquet, xlsx, or csv.");
        }
    }

    Ok(())
}

fn handle_db_error(e: TodoError) {
    let error_message = e.to_string().to_lowercase();
    let known_errors = [
        ("no such file or directory", "File not found. Check the path and try again."),
        ("constraint", "This item already exists. Please check your input."),
        ("foreign key", "Item is still in use. Ensure it is not linked elsewhere."),
        ("gdal error", "GDAL issue occurred. Check file permissions and existence."),
    ];

    for (pattern, message) in &known_errors {
        if error_message.contains(pattern) {
            println!("{}", message);
            return;
        }
    }

    println!("Unexpected error occurred: {}. Please check the logs.", e);
}

