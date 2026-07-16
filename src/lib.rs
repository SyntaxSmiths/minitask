use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::path::PathBuf;

#[cfg(feature = "gui")]
use command_fds::{CommandFdExt, FdMapping};
#[cfg(feature = "gui")]
use std::process::{Command, Stdio};

#[cfg(all(unix, feature = "gui"))]
use std::os::fd::OwnedFd;

#[cfg(all(unix, feature = "gui"))]
use std::os::unix::net::UnixStream;

#[cfg(all(unix, feature = "gui"))]
use tokio::net::UnixStream as TokioUnixStream;

pub mod mcp;

#[cfg(feature = "gui")]
const EMBEDDED_GUI_JS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/gui.js"));

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

/// Commands enum for both CLI and MCP use
#[derive(Debug, Clone, Serialize, Deserialize, Subcommand)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum Commands {
    /// Print detailed help for all commands and subcommands
    LongHelp,
    
    /// Return tasks from the current task file. Use filters to narrow the result set before doing
    /// follow-up actions such as claim, show, edit, add, or delete.
    List {
        /// Only return tasks whose `state` exactly matches this value, for example `todo`,
        /// `in-progress`, or `done`.
        #[arg(long)]
        #[serde(default)]
        state: Option<String>,

        /// Only return tasks that belong to the given epic name.
        #[arg(long)]
        #[serde(default)]
        epic: Option<String>,

        /// Include full task content and metadata instead of only a short summary.
        #[arg(long, default_value_t = false)]
        #[serde(default)]
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
        #[serde(default = "default_state")]
        state: String,

        /// If set, only consider tasks that belong to this epic.
        #[arg(long)]
        #[serde(default)]
        epic: Option<String>,
    },
}

fn default_state() -> String {
    "todo".to_string()
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
}

/// Container for all tasks in the file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskFile {
    /// List of all tasks
    #[serde(default)]
    pub tasks: Vec<Task>,
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
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn save(&self, file: &mut File) -> Result<(), Error> {
        file.rewind()?;
        file.set_len(0)?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn new_task(&mut self, content: &str) -> Result<Task, Error> {
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

        Ok(new_task)
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

    /// Show a task by ID
    pub fn handle_show(&self, task_id: &str) -> Result<&Task, Error> {
        let task_id = normalize_task_id(task_id);
        self.select(&task_id)
    }

    /// Create a new task
    pub fn handle_new(&mut self, content: &str) -> Result<Task, Error> {
        self.new_task(content)
    }

    /// Edit task state
    pub fn handle_edit_state(&mut self, task_id: &str, state: &str) -> Result<(), Error> {
        let task_id = normalize_task_id(task_id);
        let task = self.select_mut(&task_id)?;
        task.state = state.to_string();
        Ok(())
    }

    /// Edit task content
    pub fn handle_edit_content(&mut self, task_id: &str, content: &str) -> Result<(), Error> {
        let task_id = normalize_task_id(task_id);
        let task = self.select_mut(&task_id)?;
        task.content = content.to_string();
        Ok(())
    }

    /// Add content to task
    pub fn handle_add_content(&mut self, task_id: &str, content: &str) -> Result<(), Error> {
        let task_id = normalize_task_id(task_id);
        let task = self.select_mut(&task_id)?;
        task.content.push_str(content);
        Ok(())
    }

    /// Add dependency to task
    pub fn handle_add_depends_on(&mut self, task_id: &str, depends_on: &str) -> Result<(), Error> {
        let task_id = normalize_task_id(task_id);
        let depends_on = normalize_task_id(depends_on);
        // Verify dependency exists
        let _ = self.select(&depends_on)?;
        let task = self.select_mut(&task_id)?;
        if !task.depends_on.contains(&depends_on) {
            task.depends_on.push(depends_on.clone());
        }
        Ok(())
    }

    /// Add epic to task
    pub fn handle_add_epic(&mut self, task_id: &str, epic: &str) -> Result<(), Error> {
        let task_id = normalize_task_id(task_id);
        let task = self.select_mut(&task_id)?;
        if !task.epic.contains(&epic.to_string()) {
            task.epic.push(epic.to_string());
        }
        Ok(())
    }

    /// Remove dependency from task
    pub fn handle_del_depends_on(&mut self, task_id: &str, depends_on: &str) -> Result<(), Error> {
        let task_id = normalize_task_id(task_id);
        let task = self.select_mut(&task_id)?;
        task.depends_on.retain(|d| d != depends_on);
        Ok(())
    }

    /// Remove epic from task
    pub fn handle_del_epic(&mut self, task_id: &str, epic: &str) -> Result<(), Error> {
        let task_id = normalize_task_id(task_id);
        let task = self.select_mut(&task_id)?;
        task.epic.retain(|e| e != epic);
        Ok(())
    }

    /// Claim next available task
    pub fn handle_claim(&mut self, new_state: &str, from_state: &str, epic_filter: Option<&str>) -> Result<&Task, Error> {
        // Find first task matching filters with no blocking dependencies
        let claimable_task = self.tasks.iter().find(|t| {
            // Match state
            let state_match = t.state == from_state;

            // Match epic if specified
            let epic_match = epic_filter.is_none_or(|e| t.epic.contains(&e.to_string()));

            // Check dependencies are not blocking
            let deps_satisfied = t.depends_on.iter().all(|dep_id| {
                self.tasks
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
        let task = self.tasks
            .iter_mut()
            .find(|t| t.name == task_id)
            .unwrap();

        task.state = new_state.to_string();
        
        // Return reference to the updated task
        Ok(self.tasks.iter().find(|t| t.name == task_id).unwrap())
    }

    /// List tasks with filters
    pub fn handle_list(&self, state_filter: Option<&str>, epic_filter: Option<&str>) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|task| {
                let state_match = state_filter.is_none_or(|s| task.state == s);
                let epic_match = epic_filter.is_none_or(|e| task.epic.contains(&e.to_string()));
                state_match && epic_match
            })
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
pub fn normalize_task_id(id: &str) -> String {
    if id.parse::<usize>().is_ok() {
        format!("TASK-{}", id)
    } else {
        id.to_string()
    }
}



pub fn serve_mcp() -> Result<(), Error> {
    use rmcp::service::ServiceExt;
    
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;

    runtime.block_on(async {
        let handler = crate::mcp::MinitaskHandler::new();
        let server = handler
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|e| Error::Io(io::Error::other(e.to_string())))?;
        
        server
            .waiting()
            .await
            .map_err(|e| Error::Io(io::Error::other(e.to_string())))?;
        
        Ok(())
    })
}

#[cfg(all(unix, feature = "gui"))]
pub fn serve_gui(task_file: PathBuf) -> Result<(), Error> {
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
        use rmcp::service::ServiceExt;
        
        let mut child = child.spawn()?;
        let tokio_stream = TokioUnixStream::from_std(mcp_stream)?;
        let (read_half, write_half) = tokio_stream.into_split();
        
        let handler = crate::mcp::MinitaskHandler::new();
        let server = handler
            .serve((read_half, write_half))
            .await
            .map_err(|e| Error::Io(io::Error::other(e.to_string())))?;
        
        let wait_child = tokio::task::spawn_blocking(move || child.wait());

        tokio::select! {
            serve_result = server.waiting() => {
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

#[cfg(all(not(unix), feature = "gui"))]
pub fn serve_gui(_task_file: PathBuf) -> Result<(), Error> {
    Err(io::Error::other("--gui is only supported on unix").into())
}

#[cfg(not(feature = "gui"))]
pub fn serve_gui(_task_file: PathBuf) -> Result<(), Error> {
    Err(io::Error::other("--gui feature not enabled").into())
}

