
# Yawmak Todo CLI

A lightweight command-line interface (CLI) for managing your todo tasks. This tool allows you to add tasks, list them, mark them as done, and manage categories, tags, and priorities. It also supports importing and exporting data in various formats.

## Features

- **Add a new todo task** with an optional due date, category, tags, and priority.
- **List all todo tasks**, showing their status, due date, priority, and associated categories and tags.
- **Mark a todo task as done**, recording the completion date.
- **Update existing tasks**, including changing the task details, category, tags, and priority.
- **Manage categories and tags**: add, delete, and list categories and tags.
- **Search for tasks** based on various criteria like name, tags, and categories.
- **Import data** from JSON, Parquet, Excel, or CSV files.
- **Export data** to JSON, Parquet, Excel, or CSV files.
- **Shell Autocomplete** for bash, zsh, fish, and PowerShell.

## Installation

### Prerequisites

- **Rust**: Ensure that you have [Rust](https://www.rust-lang.org/tools/install) installed on your system.
- **DuckDB**: Install DuckDB using Homebrew:

```bash
brew install duckdb
```

## Cloning the Repository

Clone the repository to your local machine:

```bash
git clone https://github.com/unixtime/yawmak.git
cd yawmak
```

## Building the Project

Build the project in release mode:

```bash
cargo build --release
```

## Copy the binary to local path

Example:

```bash
cp ./target/release/yawmak ~/bin or /usr/local/bin
```

The database is located/stored in `~/.yawmak/db` - You can modify the code to change the location if needed.

## Usage

### Display Help

To display help and see available commands:

```bash
yawmak --help
```

### Add a New Todo

To add a new todo task, provide the task description. You can also add an optional due date, category, tags, and priority:

```bash
yawmak add "Buy groceries" "2024-09-01" --category "Personal" --tags "urgent,food" --priority 2
```

### List All Todos

To list all todo tasks:

```bash
yawmak list
```

### List Only Completed Tasks

To list all completed tasks:

```bash
yawmak list --done-only
```

### Mark a Todo as Done

To mark a todo task as done, provide the task ID:

```bash
yawmak done 1
```

### Update an Existing Todo

To update a todo task's details:

```bash
yawmak update 1 --task "Buy fruits" --due-date "2024-09-02" --category "Personal" --tags "food" --priority 1 --undone
```

### Search Tasks

To search for tasks by name, tag, or category:

```bash
yawmak search "groceries"
```

### Manage Categories

#### Add a New Category

```bash
yawmak add-category "Work"
```

#### Delete a Category

```bash
yawmak delete-category "Personal"
```

#### List All Categories

```bash
yawmak list-categories
```

### Manage Tags

#### Add a New Tag

```bash
yawmak add-tag "important"
```

#### Delete a Tag

```bash
yawmak delete-tag "urgent"
```

#### List All Tags

```bash
yawmak list-tags
```

### Import Data

To import data from a file, specify the format (json, parquet, xlsx, or csv) and the file path:

```bash
yawmak import json data.json
```

You can also specify import strategies when importing Excel, JSON, or Parquet files:
- `skip`: Skip importing existing tasks.
- `remove`: Remove existing tasks before importing.
- `upsert`: Update existing tasks with the imported data.

Example using `upsert` strategy:

```bash
yawmak import xlsx data.xlsx --strategy upsert
```

### Export Data

To export data to a file, specify the format (json, parquet, xlsx, or csv) and the file path:

```bash
yawmak export json export.json
```

### Shell Autocomplete

To generate shell completion scripts for your shell:

For Bash:
```bash
yawmak completion bash
```

For Zsh:
```bash
yawmak completion zsh
```

For Fish:
```bash
yawmak completion fish
```

For PowerShell:
```bash
yawmak completion powershell
```

### Example Workflow

Add Tasks

```bash
yawmak add "Buy groceries" "2024-09-01" --category "Personal" --tags "urgent,food" --priority 2
yawmak add "Prepare presentation" "2024-09-03" --category "Work" --tags "important" --priority 1
```

List Tasks

```bash
yawmak list
```

Update a Task

```bash
yawmak update 1 --tags "urgent,shopping"
```

Mark a Task as Done

```bash
yawmak done 1
```

List Only Completed Tasks

```bash
yawmak list --done-only
```

## Development

If you want to contribute or modify the project, follow these steps:

### Clone the Repository:

```bash
git clone https://github.com/unixtime/yawmak.git
cd yawmak
```

### Update Dependencies: Ensure that Cargo.toml includes the necessary dependencies:

```toml
[dependencies]
clap = "4.0"
duckdb = "0.7"
chrono = "0.4"
prettytable-rs = "0.10.0"
```

### Build the Project:

```bash
cargo build --release
```

### Run the Application:

```bash
yawmak
```

## License

This project is licensed under the MIT License. See the LICENSE file for more details.

## Acknowledgments

- Rust for the awesome programming language.
- DuckDB for the embedded database.
- Chrono for date and time handling in Rust.
- Clap for command-line argument parsing.
- PrettyTable-rs for formatted table output.

### Key Updates:

1. **New Features**: Added details about import/export functionality and shell autocomplete.
2. **Usage Instructions**: Updated commands to include new functionalities for import/export and autocomplete.
3. **Example Workflow**: Expanded with examples showing how to add, update, mark as done, and manage categories/tags.
4. **Development Section**: Updated dependencies to include `prettytable-rs` for table formatting.
