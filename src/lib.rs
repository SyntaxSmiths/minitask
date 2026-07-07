use clap::{Parser, Subcommand};
use clap_mcp::{ClapMcp, ClapMcpToolError, ClapMcpToolOutput, McpListen, ServeMcpBuilder};
use command_fds::{CommandFdExt, FdMapping};
use file_lock::{FileLock, FileOptions};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Mutex, PoisonError};

#[cfg(unix)]
use std::os::fd::OwnedFd;

#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(unix)]
use tokio::net::UnixStream as TokioUnixStream;

include!(concat!(env!("OUT_DIR"), "/embedded_gui.rs"));

pub fn print_full_help(cmd: &mut clap::Command) {
    print_full_help_inner(cmd, "");
}

fn print_full_help_inner(cmd: &mut clap::Command, path: &str) {
    let name = cmd.get_name();

    // Build full command path (e.g. "myapp foo bar")
    let full_path = if path.is_empty() {
        name.to_string()
    } else {
        format!("{} {}", path, name)
    };

    // 👇 Header so it's obvious which command we're printing
    println!("========== {} ==========\n", full_path);

    cmd.print_long_help().unwrap();
    println!("\n");

    // Recurse into subcommands
    for sub in cmd.get_subcommands_mut() {
        print_full_help_inner(sub, &full_path);
    }
}

/// Task management CLI tool
#[derive(Parser, Debug, ClapMcp)]
#[clap_mcp(reinvocation_safe, parallel_safe = false, stateful)]
#[command(name = "minitask", about = "A simple task management tool", long_about = None, arg_required_else_help = true)]
pub struct Cli {
    /// Start the GTK GUI via gjs.
    #[arg(long, global = true, action = clap::ArgAction::SetTrue)]
    pub gui: bool,

    /// Start the MCP server over stdio.
    #[arg(long, global = true, action = clap::ArgAction::SetTrue)]
    pub mcp: bool,

    // Custom long help
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub long_help: bool,

    /// Path to the tasks file
    #[arg(long, global = true, default_value = "tasks.toml")]
    pub file: PathBuf,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

pub struct State {
    pub file: PathBuf,
    pub cli_out: bool,
}

#[derive(Subcommand, Debug, Serialize, Deserialize, ClapMcp)]
#[clap_mcp(reinvocation_safe)]
#[clap_mcp_output_from_with_state = "mcp_pass"]
#[clap_mcp_state_type = "Mutex<State>"]
pub enum Commands {
    /// Switch the active tasks file for this invocation. Use this before other first minitask action and
    /// set it to a preferred tasks tracking file. Always use the full file path.
    File { path: PathBuf },
    /// Return tasks from the current task file. Use filters to narrow the result set before doing
    /// follow-up actions such as claim, show, edit, add, or delete.
    List {
        /// Only return tasks whose `state` exactly matches this value, for example `todo`,
        /// `in-progress`, or `done`.
        #[arg(long)]
        state: Option<String>,

        /// Only return tasks that belong to the given epic name.
        #[arg(long)]
        epic: Option<String>,

        /// Include full task content and metadata instead of only a short summary.
        #[arg(long, default_value_t = false)]
        verbose: bool,
    },
    /// Return one task by ID. Use this when the agent already knows the task ID and needs the
    /// current full task record before deciding what to change.
    Show {
        /// Task identifier such as `TASK-3`. Plain numbers are also accepted and normalized.
        #[arg(long)]
        task_id: String,
    },
    /// Create a new task. The provided content becomes the full task body.
    New {
        /// Task body text. Pass `-` to read the content from stdin.
        #[arg(long)]
        content: String,
    },
    /// Change the `state` field of an existing task.
    EditState {
        /// Task identifier such as `TASK-3`. Plain numbers are also accepted and normalized.
        #[arg(long)]
        task_id: String,
        /// New state value to write into the task, for example `todo`, `in-progress`, or `done`.
        #[arg(long)]
        state: String,
    },
    /// Replace the full text content of an existing task.
    EditContent {
        /// Task identifier such as `TASK-3`. Plain numbers are also accepted and normalized.
        #[arg(long)]
        task_id: String,
        /// New full task body that replaces the current content.
        #[arg(long)]
        content: String,
    },
    /// Append additional text to the end of an existing task's content.
    AddContent {
        /// Task identifier such as `TASK-3`. Plain numbers are also accepted and normalized.
        #[arg(long)]
        task_id: String,
        /// Text to append to the existing task body.
        #[arg(long)]
        content: String,
    },
    /// Add a dependency so this task records that it depends on another task.
    AddDependsOn {
        /// Task identifier such as `TASK-3`. Plain numbers are also accepted and normalized.
        #[arg(long)]
        task_id: String,
        /// Identifier of the task this task depends on.
        #[arg(long)]
        depends_on: String,
    },
    /// Attach an epic label to an existing task.
    AddEpic {
        /// Task identifier such as `TASK-3`. Plain numbers are also accepted and normalized.
        #[arg(long)]
        task_id: String,
        /// Epic name to add to the task.
        #[arg(long)]
        epic: String,
    },
    /// Remove one dependency from an existing task.
    DelDependsOn {
        /// Task identifier such as `TASK-3`. Plain numbers are also accepted and normalized.
        #[arg(long)]
        task_id: String,
        /// Dependency task identifier to remove.
        #[arg(long)]
        depends_on: String,
    },
    /// Remove one epic label from an existing task.
    DelEpic {
        /// Task identifier such as `TASK-3`. Plain numbers are also accepted and normalized.
        #[arg(long)]
        task_id: String,
        /// Epic name to remove from the task.
        #[arg(long)]
        epic: String,
    },
    /// Move the next matching task from one state to another and return the claimed task. Use this
    /// to reserve work before editing it further.
    Claim {
        /// State to assign to the claimed task, for example `in-progress`.
        #[arg(long)]
        new_state: String,

        /// Only consider tasks currently in this source state. Defaults to `todo`.
        #[arg(long, default_value = "todo")]
        state: String,

        /// If set, only consider tasks that belong to this epic.
        #[arg(long)]
        epic: Option<String>,
    },
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    Io(std::io::Error),
    Poison,
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}
impl clap_mcp::IntoClapMcpToolError for Error {
    fn into_tool_error(self) -> ClapMcpToolError {
        ClapMcpToolError::text(format!("{self:?}"))
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Error::Poison
    }
}

/// Represents a single task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier (e.g., "TASK-0")
    pub name: String,

    /// Current state of the task (e.g., "todo", "in-progress", "done")
    pub state: String,

    /// List of task IDs this task depends on
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// List of epics this task belongs to
    #[serde(default)]
    pub epic: Vec<String>,

    /// Task description and details
    pub content: String,
}
impl Task {
    pub fn new(
        name: &str,
        state: &str,
        depends_on: Vec<&str>,
        epic: Vec<&str>,
        content: &str,
    ) -> Task {
        Self {
            name: name.to_string(),
            state: state.to_string(),
            depends_on: depends_on.into_iter().map(String::from).collect(),
            epic: epic.into_iter().map(String::from).collect(),
            content: content.to_string(),
        }
    }

    pub fn select(&self, state: Option<&str>, epic: Option<&str>) -> bool {
        match (state, epic) {
            (Some(state), Some(epic)) => {
                if self.state == state && self.epic.contains(&epic.to_string()) {
                    return true;
                }
                false
            }
            (Some(state), None) => self.state == state,
            (None, Some(epic)) => self.epic.contains(&epic.to_string()),
            (None, None) => true,
        }
    }

    pub fn show(&self, cli_out: bool) -> Result<TaskFile, Error> {
        // Find the task
        if cli_out {
            print_task_verbose(self);
        }

        Ok(TaskFile {
            tasks: vec![self.clone()],
        })
    }
}

/// Container for all tasks in the file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskFile {
    /// List of all tasks
    #[serde(default)]
    pub tasks: Vec<Task>,
}

impl clap_mcp::IntoClapMcpResult for TaskFile {
    fn into_tool_result(self) -> std::result::Result<ClapMcpToolOutput, ClapMcpToolError> {
        Ok(ClapMcpToolOutput::Structured(
            serde_json::to_value(self)
                .map_err(|error| ClapMcpToolError::text(format!("{error:?}")))?,
        ))
    }
}

impl TaskFile {
    /// Loads tasks from a TOML file. Creates an empty file if it doesn't exist.
    ///
    /// # Arguments
    /// * `file` - File handle
    ///
    /// # Returns
    /// * `Result<TaskFile, Error>` - The loaded TaskFile or an error
    pub fn load(file: &mut File) -> Result<TaskFile, Error> {
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
    /// * `Result<TaskFile, Error>` - Success or an error
    pub fn save(&self, file: &mut File) -> Result<TaskFile, Error> {
        file.rewind()?;
        file.set_len(0)?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        file.write_all(content.as_bytes())?;
        Ok((*self).clone())
    }

    pub fn new(&mut self, content: &str, cli_out: bool) -> Result<TaskFile, Error> {
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
        let next_id = self
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
        let new_task = Task::new(&task_name, "todo", vec![], vec![], &task_content);

        self.tasks.push(new_task.clone());

        if cli_out {
            println!("Created {}", task_name);
        }

        Ok(TaskFile {
            tasks: vec![new_task],
        })
    }

    pub fn select(&self, task_id: &str) -> Result<&Task, Error> {
        self.tasks
            .iter()
            .find(|t| t.name == task_id)
            .ok_or_else(|| {
                Error::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Task '{}' not found", task_id),
                ))
            })
    }

    pub fn select_mut(&mut self, task_id: &str) -> Result<&mut Task, Error> {
        self.tasks
            .iter_mut()
            .find(|t| t.name == task_id)
            .ok_or_else(|| {
                Error::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Task '{}' not found", task_id),
                ))
            })
    }

    pub fn select_many(&self, state: Option<&str>, epic: Option<&str>) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.select(state, epic))
            .collect()
    }

    pub fn select_many_mut(&mut self, state: Option<&str>, epic: Option<&str>) -> Vec<&mut Task> {
        self.tasks
            .iter_mut()
            .filter(|t| t.select(state, epic))
            .collect()
    }
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

pub fn cli_pass(cli: Cli) -> Result<TaskFile, Error> {
    let options = FileOptions::new().write(true).read(true).create(true);
    let mut file_lock = FileLock::lock(&cli.file, false, options)?;
    let mut tasks_file = TaskFile::load(&mut file_lock.file)?;
    let mut state = State {
        cli_out: true,
        file: cli.file,
    };

    let result = command_pass(
        &mut tasks_file,
        cli.command.ok_or(std::io::Error::from_raw_os_error(22))?,
        &mut state,
    );
    if result.is_ok() {
        tasks_file.save(&mut file_lock.file)?;
    }

    result
}

pub fn serve_mcp(state: std::sync::Arc<Mutex<State>>) -> Result<(), Error> {
    ServeMcpBuilder::for_cli_with_state::<Cli>(McpListen::Stdio, state)
        .serve_blocking()
        .map_err(|error| io::Error::other(error.to_string()))?;
    Ok(())
}

#[cfg(unix)]
pub fn serve_gui(state: std::sync::Arc<Mutex<State>>, task_file: PathBuf) -> Result<(), Error> {
    let (gui_stream, mcp_stream) = UnixStream::pair()?;
    let gui_fd: OwnedFd = gui_stream.into();
    mcp_stream.set_nonblocking(true)?;

    let mut child = Command::new("gjs");
    child
        .arg("-c")
        .arg(EMBEDDED_GUI_JS)
        .env("MINITASK_GUI_SOCKET_FD", "3")
        .env("MINITASK_GUI_TASK_FILE", task_file)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    child
        .fd_mappings(vec![FdMapping {
            parent_fd: gui_fd,
            child_fd: 3,
        }])
        .map_err(|error| io::Error::other(error.to_string()))?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;

    runtime.block_on(async move {
        let mut child = child.spawn()?;
        let tokio_stream = TokioUnixStream::from_std(mcp_stream)?;
        let (read_half, write_half) = tokio_stream.into_split();
        let serve = ServeMcpBuilder::for_cli_with_state::<Cli>(McpListen::Stdio, state)
            .stdio_io(read_half, write_half)
            .serve();
        let wait_child = tokio::task::spawn_blocking(move || child.wait());

        tokio::pin!(serve);

        tokio::select! {
            serve_result = &mut serve => {
                serve_result.map_err(|error| io::Error::other(error.to_string()))?;
            }
            wait_result = wait_child => {
                let status = wait_result.map_err(|error| io::Error::other(error.to_string()))??;
                if !status.success() {
                    return Err(io::Error::other(format!("gjs exited with status {status}")).into());
                }

                return Ok(());
            }
        }

        Ok(())
    })
}

#[cfg(not(unix))]
pub fn serve_gui(_state: std::sync::Arc<Mutex<State>>, _task_file: PathBuf) -> Result<(), Error> {
    Err(io::Error::other("--gui is only supported on unix").into())
}

fn mcp_pass(command: Commands, state: &Mutex<State>) -> Result<TaskFile, Error> {
    let mut state = state.lock()?;

    let options = FileOptions::new().write(true).read(true).create(true);
    let mut filelock = FileLock::lock(&state.file, false, options)?;
    let mut tasks_file = TaskFile::load(&mut filelock.file)?;

    let result = command_pass(&mut tasks_file, command, &mut state);
    if result.is_ok() {
        tasks_file.save(&mut filelock.file)?;
    }

    result
}

fn command_pass(
    mut tasks_file: &mut TaskFile,
    command: Commands,
    app_state: &mut State,
) -> Result<TaskFile, Error> {
    match command {
        Commands::List {
            state,
            epic,
            verbose,
        } => handle_list(
            &mut tasks_file,
            state.as_deref(),
            epic.as_deref(),
            verbose,
            app_state.cli_out,
        ),
        Commands::Show { task_id } => tasks_file.select(&task_id)?.show(app_state.cli_out),
        Commands::New { content } => tasks_file.new(&content, app_state.cli_out),
        Commands::EditState { task_id, state } => {
            let task_id = normalize_task_id(&task_id);
            let task = tasks_file.select_mut(&task_id)?;
            task.state = state.clone();
            if app_state.cli_out {
                println!("Updated {} state to {}", task_id, state);
            }
            Ok(TaskFile {
                tasks: vec![task.clone()],
            })
        }
        Commands::EditContent { task_id, content } => {
            let task_id = normalize_task_id(&task_id);
            let task = tasks_file.select_mut(&task_id)?;
            task.content = content.clone();
            if app_state.cli_out {
                println!("Updated {} content", task_id);
            }

            Ok(TaskFile {
                tasks: vec![task.clone()],
            })
        }
        Commands::AddContent { task_id, content } => {
            let task_id = normalize_task_id(&task_id);
            let task = tasks_file.select_mut(&task_id)?;
            task.content.push_str(&content);

            if app_state.cli_out {
                println!("Appended content to {}", task_id);
            }

            Ok(TaskFile {
                tasks: vec![task.clone()],
            })
        }
        Commands::AddDependsOn {
            task_id,
            depends_on,
        } => {
            let task_id = normalize_task_id(&task_id);
            let depends_on = normalize_task_id(&depends_on);
            let _ = tasks_file.select(&depends_on)?;
            let task = tasks_file.select_mut(&task_id)?;
            if !task.depends_on.contains(&depends_on) {
                task.depends_on.push(depends_on.clone());
            }

            if app_state.cli_out {
                println!("Added dependency {} to {}", depends_on, task_id);
            }

            Ok(TaskFile {
                tasks: vec![task.clone()],
            })
        }
        Commands::AddEpic { task_id, epic } => {
            let task_id = normalize_task_id(&task_id);
            let task = tasks_file.select_mut(&task_id)?;

            // Prevent duplicates
            if !task.epic.contains(&epic) {
                task.epic.push(epic.clone());
            }

            if app_state.cli_out {
                println!("Added epic {} to {}", epic, task_id);
            }

            Ok(TaskFile {
                tasks: vec![task.clone()],
            })
        }
        Commands::DelDependsOn {
            task_id,
            depends_on,
        } => {
            let task_id = normalize_task_id(&task_id);
            let task = tasks_file.select_mut(&task_id)?;

            // Remove epic
            task.depends_on.retain(|d| d != &depends_on);

            if app_state.cli_out {
                println!("Removed dependency {} from {}", depends_on, task_id);
            }

            Ok(TaskFile {
                tasks: vec![task.clone()],
            })
        }
        Commands::DelEpic { task_id, epic } => {
            let task_id = normalize_task_id(&task_id);
            let task = tasks_file.select_mut(&task_id)?;

            // Remove epic
            task.epic.retain(|e| e != &epic);

            if app_state.cli_out {
                println!("Removed epic {} from {}", epic, task_id);
            }

            Ok(TaskFile {
                tasks: vec![task.clone()],
            })
        }
        Commands::Claim {
            new_state,
            state,
            epic,
        } => handle_claim(
            &mut tasks_file,
            &new_state,
            &state,
            epic.as_deref(),
            app_state.cli_out,
        ),
        Commands::File { path } => {
            app_state.file = PathBuf::from(path);
            Ok(TaskFile { tasks: vec![] })
        }
    }
}

/// Handles the claim command
fn handle_claim(
    task_file: &mut TaskFile,
    new_state: &str,
    from_state: &str,
    epic_filter: Option<&str>,
    cli_out: bool,
) -> Result<TaskFile, Error> {
    // Find first task matching filters with no blocking dependencies
    let claimable_task = task_file.tasks.iter().find(|t| {
        // Match state
        let state_match = t.state == from_state;

        // Match epic if specified
        let epic_match = epic_filter.is_none_or(|e| t.epic.contains(&e.to_string()));

        // Check dependencies are not blocking
        let deps_satisfied = t.depends_on.iter().all(|dep_id| {
            task_file
                .tasks
                .iter()
                .find(|dt| dt.name == *dep_id)
                .is_none_or(|dt| dt.state == "done")
        });

        state_match && epic_match && deps_satisfied
    });

    let task_id = match claimable_task {
        Some(t) => t.name.clone(),
        None => {
            return Err(
                io::Error::new(io::ErrorKind::NotFound, "No available tasks to claim").into(),
            );
        }
    };

    // Update the task state
    let task = task_file
        .tasks
        .iter_mut()
        .find(|t| t.name == task_id)
        .unwrap();

    task.state = new_state.to_string();

    if cli_out {
        println!("Claimed {} and moved to {}", task_id, new_state);
    }

    Ok(TaskFile {
        tasks: vec![task.clone()],
    })
}

/// Handles the list command
fn handle_list(
    task_file: &mut TaskFile,
    state_filter: Option<&str>,
    epic_filter: Option<&str>,
    verbose: bool,
    cli_out: bool,
) -> Result<TaskFile, Error> {
    // Filter tasks
    let mut filtered_tasks: Vec<Task> = task_file
        .tasks
        .iter()
        .filter(|task| {
            let state_match = state_filter.is_none_or(|s| task.state == s);
            let epic_match = epic_filter.is_none_or(|e| task.epic.contains(&e.to_string()));
            state_match && epic_match
        })
        .map(|task| task.clone())
        .collect();

    if cli_out {
        if verbose {
            // Verbose output
            for task in &filtered_tasks {
                print_task_verbose(task);
            }
        } else {
            // Normal output - show task ID and first line of content
            for task in &filtered_tasks {
                let first_line = task.content.lines().next().unwrap_or("");
                println!("{}: {}", task.name, first_line);
            }
        }
    }

    if !verbose {
        filtered_tasks.iter_mut().for_each(|task| {
            task.content = String::new();
        });
    }

    Ok(TaskFile {
        tasks: filtered_tasks,
    })
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

#[cfg(test)]
mod tests;
