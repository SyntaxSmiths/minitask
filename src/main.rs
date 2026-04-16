use clap::{CommandFactory, Parser, Subcommand};
use file_lock::{FileLock, FileOptions};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::path::PathBuf;

/// Task management CLI tool
#[derive(Parser, Debug)]
#[command(name = "minitask")]
#[command(about = "A simple task management tool", long_about = None, arg_required_else_help = true)]
struct Cli {
    /// Output results as JSON
    #[arg(long, global = true)]
    json_out: bool,

    /// Accept input as JSON from stdin
    #[arg(long, global = true)]
    json_in: bool,

    /// Path to the tasks file
    #[arg(long, global = true, default_value = "tasks.toml")]
    file: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
enum Commands {
    /// List tasks
    List {
        /// Filter by state
        #[arg(long)]
        state: Option<String>,

        /// Filter by epic
        #[arg(long)]
        epic: Option<String>,

        /// Show verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Show a specific task
    Show {
        /// Task ID to show
        task_id: String,

        /// Show verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Create a new task
    New {
        /// Task content (use "-" to read from stdin)
        content: String,
    },
    /// Edit task properties
    Edit {
        #[command(subcommand)]
        edit_command: EditCommands,
    },
    /// Add to task properties
    Add {
        #[command(subcommand)]
        add_command: AddCommands,
    },
    /// Delete from task properties
    Del {
        #[command(subcommand)]
        del_command: DelCommands,
    },
    /// Claim the next available task
    Claim {
        /// New state for the claimed task
        new_state: String,

        /// Filter by source state
        #[arg(long, default_value = "todo")]
        state: String,

        /// Filter by epic
        #[arg(long)]
        epic: Option<String>,
    },
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
enum EditCommands {
    /// Edit task state
    State {
        /// Task ID
        task_id: String,
        /// New state
        state: String,
    },
    /// Edit task content
    Content {
        /// Task ID
        task_id: String,
        /// New content
        content: String,
    },
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
enum AddCommands {
    /// Append to task content
    Content {
        /// Task ID
        task_id: String,
        /// Content to append
        content: String,
    },
    /// Add a dependency
    DependsOn {
        /// Task ID
        task_id: String,
        /// Dependency task ID
        depends_on: String,
    },
    /// Add task to an epic
    Epic {
        /// Task ID
        task_id: String,
        /// Epic name
        epic: String,
    },
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
enum DelCommands {
    /// Remove a dependency
    DependsOn {
        /// Task ID
        task_id: String,
        /// Dependency task ID to remove
        depends_on: String,
    },
    /// Remove task from an epic
    Epic {
        /// Task ID
        task_id: String,
        /// Epic name to remove
        epic: String,
    },
}

/// Represents a single task
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    /// Unique task identifier (e.g., "TASK-0")
    name: String,

    /// Current state of the task (e.g., "todo", "in-progress", "done")
    state: String,

    /// List of task IDs this task depends on
    #[serde(default)]
    depends_on: Vec<String>,

    /// List of epics this task belongs to
    #[serde(default)]
    epic: Vec<String>,

    /// Task description and details
    content: String,
}

/// Container for all tasks in the file
#[derive(Debug, Serialize, Deserialize)]
struct TaskFile {
    /// List of all tasks
    #[serde(default)]
    tasks: Vec<Task>,
}

/// Loads tasks from a TOML file. Creates an empty file if it doesn't exist.
///
/// # Arguments
/// * `file` - File handle
///
/// # Returns
/// * `io::Result<TaskFile>` - The loaded TaskFile or an error
fn load_tasks(file: &mut File) -> io::Result<TaskFile> {
    // Read and parse existing file
    let mut content = String::new();
    file.rewind()?;
    let _ = file.read_to_string(&mut content)?;
    let task_file: TaskFile =
        toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(task_file)
}

/// Saves tasks to a TOML file.
///
/// # Arguments
/// * `file` - file handle
/// * `task_file` - The TaskFile to save
///
/// # Returns
/// * `io::Result<()>` - Success or an error
fn save_tasks(file: &mut File, task_file: &TaskFile) -> io::Result<()> {
    file.rewind()?;
    file.set_len(0)?;
    let content = toml::to_string_pretty(task_file)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let _ = file.write_all(content.as_bytes())?;
    Ok(())
}

/// Normalizes a task ID by prepending "TASK-" if only a number is provided
///
/// # Arguments
/// * `id` - The task ID or number
///
/// # Returns
/// * `String` - Normalized task ID (e.g., "1" -> "TASK-1", "TASK-1" -> "TASK-1")
fn normalize_task_id(id: &str) -> String {
    if id.parse::<usize>().is_ok() {
        format!("TASK-{}", id)
    } else {
        id.to_string()
    }
}

fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();

    let options = FileOptions::new().write(true).read(true).create(true);
    let mut filelock = FileLock::lock(&cli.file, false, options)?;
    let mut tasks_file = load_tasks(&mut filelock.file)?;

    let result = match cli.command {
        Some(Commands::List {
            state,
            epic,
            verbose,
        }) => handle_list(
            &mut tasks_file,
            state.as_deref(),
            epic.as_deref(),
            verbose,
            cli.json_out,
        ),
        Some(Commands::Show { task_id, verbose }) => {
            let task_id = normalize_task_id(&task_id);
            handle_show(&mut tasks_file, &task_id, verbose, cli.json_out)
        }
        Some(Commands::New { content }) => {
            handle_new(&mut tasks_file, &content, cli.json_in, cli.json_out)
        }
        Some(Commands::Edit { edit_command }) => match edit_command {
            EditCommands::State { task_id, state } => {
                let task_id = normalize_task_id(&task_id);
                handle_edit_state(&mut tasks_file, &task_id, &state, cli.json_out)
            }
            EditCommands::Content { task_id, content } => {
                let task_id = normalize_task_id(&task_id);
                handle_edit_content(&mut tasks_file, &task_id, &content, cli.json_out)
            }
        },
        Some(Commands::Add { add_command }) => match add_command {
            AddCommands::Content { task_id, content } => {
                let task_id = normalize_task_id(&task_id);
                handle_add_content(&mut tasks_file, &task_id, &content, cli.json_out)
            }
            AddCommands::DependsOn {
                task_id,
                depends_on,
            } => {
                let task_id = normalize_task_id(&task_id);
                let depends_on = normalize_task_id(&depends_on);
                handle_add_depends_on(&mut tasks_file, &task_id, &depends_on, cli.json_out)
            }
            AddCommands::Epic { task_id, epic } => {
                let task_id = normalize_task_id(&task_id);
                handle_add_epic(&mut tasks_file, &task_id, &epic, cli.json_out)
            }
        },
        Some(Commands::Del { del_command }) => match del_command {
            DelCommands::DependsOn {
                task_id,
                depends_on,
            } => {
                let task_id = normalize_task_id(&task_id);
                let depends_on = normalize_task_id(&depends_on);
                handle_del_depends_on(&mut tasks_file, &task_id, &depends_on, cli.json_out)
            }
            DelCommands::Epic { task_id, epic } => {
                let task_id = normalize_task_id(&task_id);
                handle_del_epic(&mut tasks_file, &task_id, &epic, cli.json_out)
            }
        },
        Some(Commands::Claim {
            new_state,
            state,
            epic,
        }) => handle_claim(
            &mut tasks_file,
            &new_state,
            &state,
            epic.as_deref(),
            cli.json_out,
        ),
        _ => {
            let _ = Cli::command().print_long_help();
            std::process::exit(1);
        }
    };

    if result.is_ok() {
        save_tasks(&mut filelock.file, &mut tasks_file)?;
    }
    result
}

/// Handles the show command
fn handle_show(
    task_file: &mut TaskFile,
    task_id: &str,
    verbose: bool,
    json_out: bool,
) -> io::Result<()> {
    // Find the task
    let task = task_file
        .tasks
        .iter()
        .find(|t| t.name == task_id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Task '{}' not found", task_id),
            )
        })?;

    if json_out {
        let json = serde_json::to_string_pretty(task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{}", json);
    } else if verbose {
        print_task_verbose(task);
    } else {
        let first_line = task.content.lines().next().unwrap_or("");
        println!("{}: {}", task.name, first_line);
    }

    Ok(())
}

/// Handles the new command
fn handle_new(
    task_file: &mut TaskFile,
    content: &str,
    _json_in: bool,
    json_out: bool,
) -> io::Result<()> {
    use std::io::Read;

    // Read content from stdin if "-"
    let task_content = if content == "-" {
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        content.to_string()
    };

    // Generate unique task ID
    let next_id = task_file
        .tasks
        .iter()
        .filter_map(|t| {
            t.name
                .strip_prefix("TASK-")
                .and_then(|n| n.parse::<usize>().ok())
        })
        .max()
        .map(|n| n + 1)
        .unwrap_or(0);

    let task_name = format!("TASK-{}", next_id);

    // Create new task
    let new_task = Task {
        name: task_name.clone(),
        state: "todo".to_string(),
        depends_on: vec![],
        epic: vec![],
        content: task_content,
    };

    task_file.tasks.push(new_task.clone());

    if json_out {
        let json = serde_json::to_string_pretty(&new_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{}", json);
    } else {
        println!("Created {}", task_name);
    }

    Ok(())
}

/// Handles the edit state command
fn handle_edit_state(
    task_file: &mut TaskFile,
    task_id: &str,
    new_state: &str,
    json_out: bool,
) -> io::Result<()> {
    // Find and update the task
    let task = task_file
        .tasks
        .iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Task '{}' not found", task_id),
            )
        })?;

    task.state = new_state.to_string();
    let updated_task = task.clone();

    if json_out {
        let json = serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{}", json);
    } else {
        println!("Updated {} state to {}", task_id, new_state);
    }

    Ok(())
}

/// Handles the edit content command
fn handle_edit_content(
    task_file: &mut TaskFile,
    task_id: &str,
    new_content: &str,
    json_out: bool,
) -> io::Result<()> {
    // Find and update the task
    let task = task_file
        .tasks
        .iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Task '{}' not found", task_id),
            )
        })?;

    task.content = new_content.to_string();
    let updated_task = task.clone();

    if json_out {
        let json = serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{}", json);
    } else {
        println!("Updated {} content", task_id);
    }

    Ok(())
}

/// Handles the add content command
fn handle_add_content(
    task_file: &mut TaskFile,
    task_id: &str,
    content_to_add: &str,
    json_out: bool,
) -> io::Result<()> {
    // Find and update the task
    let task = task_file
        .tasks
        .iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Task '{}' not found", task_id),
            )
        })?;

    task.content.push_str(content_to_add);
    let updated_task = task.clone();

    if json_out {
        let json = serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{}", json);
    } else {
        println!("Appended content to {}", task_id);
    }

    Ok(())
}

/// Handles the add depends-on command
fn handle_add_depends_on(
    task_file: &mut TaskFile,
    task_id: &str,
    depends_on_id: &str,
    json_out: bool,
) -> io::Result<()> {
    // Validate both tasks exist
    if !task_file.tasks.iter().any(|t| t.name == depends_on_id) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Dependency task '{}' not found", depends_on_id),
        ));
    }

    // Find and update the task
    let task = task_file
        .tasks
        .iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Task '{}' not found", task_id),
            )
        })?;

    // Prevent duplicates
    if !task.depends_on.contains(&depends_on_id.to_string()) {
        task.depends_on.push(depends_on_id.to_string());
    }
    let updated_task = task.clone();

    if json_out {
        let json = serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{}", json);
    } else {
        println!("Added dependency {} to {}", depends_on_id, task_id);
    }

    Ok(())
}

/// Handles the del depends-on command
fn handle_del_depends_on(
    task_file: &mut TaskFile,
    task_id: &str,
    depends_on_id: &str,
    json_out: bool,
) -> io::Result<()> {
    // Find and update the task
    let task = task_file
        .tasks
        .iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Task '{}' not found", task_id),
            )
        })?;

    // Remove dependency
    task.depends_on.retain(|d| d != depends_on_id);
    let updated_task = task.clone();

    if json_out {
        let json = serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{}", json);
    } else {
        println!("Removed dependency {} from {}", depends_on_id, task_id);
    }

    Ok(())
}

/// Prints a task in verbose format

/// Handles the add epic command
fn handle_add_epic(
    task_file: &mut TaskFile,
    task_id: &str,
    epic_name: &str,
    json_out: bool,
) -> io::Result<()> {
    // Find and update the task
    let task = task_file
        .tasks
        .iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Task '{}' not found", task_id),
            )
        })?;

    // Prevent duplicates
    if !task.epic.contains(&epic_name.to_string()) {
        task.epic.push(epic_name.to_string());
    }
    let updated_task = task.clone();

    if json_out {
        let json = serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{}", json);
    } else {
        println!("Added epic {} to {}", epic_name, task_id);
    }

    Ok(())
}

/// Handles the del epic command
fn handle_del_epic(
    task_file: &mut TaskFile,
    task_id: &str,
    epic_name: &str,
    json_out: bool,
) -> io::Result<()> {
    // Find and update the task
    let task = task_file
        .tasks
        .iter_mut()
        .find(|t| t.name == task_id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Task '{}' not found", task_id),
            )
        })?;

    // Remove epic
    task.epic.retain(|e| e != epic_name);
    let updated_task = task.clone();

    if json_out {
        let json = serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{}", json);
    } else {
        println!("Removed epic {} from {}", epic_name, task_id);
    }

    Ok(())
}

/// Handles the claim command
fn handle_claim(
    task_file: &mut TaskFile,
    new_state: &str,
    from_state: &str,
    epic_filter: Option<&str>,
    json_out: bool,
) -> io::Result<()> {
    // Find first task matching filters with no blocking dependencies
    let claimable_task = task_file
        .tasks
        .iter()
        .filter(|t| {
            // Match state
            let state_match = t.state == from_state;

            // Match epic if specified
            let epic_match = epic_filter.map_or(true, |e| t.epic.contains(&e.to_string()));

            // Check dependencies are not blocking
            let deps_satisfied = t.depends_on.iter().all(|dep_id| {
                task_file
                    .tasks
                    .iter()
                    .find(|dt| dt.name == *dep_id)
                    .map_or(true, |dt| dt.state == "done")
            });

            state_match && epic_match && deps_satisfied
        })
        .next();

    let task_id = match claimable_task {
        Some(t) => t.name.clone(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No available tasks to claim",
            ));
        }
    };

    // Update the task state
    let task = task_file
        .tasks
        .iter_mut()
        .find(|t| t.name == task_id)
        .unwrap();

    task.state = new_state.to_string();
    let updated_task = task.clone();

    if json_out {
        let json = serde_json::to_string_pretty(&updated_task)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{}", json);
    } else {
        println!("Claimed {} and moved to {}", task_id, new_state);
    }

    Ok(())
}

fn print_task_verbose(task: &Task) {
    println!("Task: {}", task.name);
    println!("State: {}", task.state);
    if !task.depends_on.is_empty() {
        println!("Depends on: {}", task.depends_on.join(", "));
    }
    if !task.epic.is_empty() {
        println!("Epics: {}", task.epic.join(", "));
    }
    println!("Content:\n{}", task.content);
    println!();
}

/// Handles the list command
fn handle_list(
    task_file: &mut TaskFile,
    state_filter: Option<&str>,
    epic_filter: Option<&str>,
    verbose: bool,
    json_out: bool,
) -> io::Result<()> {
    // Filter tasks
    let filtered_tasks: Vec<&Task> = task_file
        .tasks
        .iter()
        .filter(|task| {
            let state_match = state_filter.map_or(true, |s| task.state == s);
            let epic_match = epic_filter.map_or(true, |e| task.epic.contains(&e.to_string()));
            state_match && epic_match
        })
        .collect();

    if json_out {
        // Output as JSON
        let json = serde_json::to_string_pretty(&filtered_tasks)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{}", json);
    } else if verbose {
        // Verbose output
        for task in filtered_tasks {
            print_task_verbose(task);
        }
    } else {
        // Normal output - show task ID and first line of content
        for task in filtered_tasks {
            let first_line = task.content.lines().next().unwrap_or("");
            println!("{}: {}", task.name, first_line);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
