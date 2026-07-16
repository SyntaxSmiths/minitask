//! MCP server handler and tool implementations

use super::tools::*;
use log::{debug, error, info};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::schemars::JsonSchema;
use rmcp::{schemars, tool, tool_router};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Parameter Types
// ============================================================================

fn default_task_file() -> PathBuf {
    PathBuf::from("tasks.toml")
}

fn default_claim_state() -> String {
    "todo".to_string()
}

/// Parameters for list tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListParams {
    /// Path to the tasks file
    ///
    /// Defaults to "tasks.toml" if omitted. Use absolute paths for clarity.
    /// The file will be created if it doesn't exist.
    #[serde(default = "default_task_file")]
    pub file: PathBuf,

    /// Only return tasks whose `state` exactly matches this value
    ///
    /// Common states: "todo", "in-progress", "review", "done", "blocked"
    /// Omit to return tasks in all states.
    pub state: Option<String>,

    /// Only return tasks that belong to the given epic name
    ///
    /// Epics are labels for grouping related tasks. Omit to return tasks
    /// from all epics.
    pub epic: Option<String>,

    /// Include full task content and metadata instead of only a short summary
    ///
    /// When false (default), only returns task ID and first line of content.
    /// Set to true when you need the complete task details including
    /// dependencies, epics, and full content.
    #[serde(default)]
    pub verbose: bool,
}

/// Parameters for show tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ShowParams {
    /// Path to the tasks file
    #[serde(default = "default_task_file")]
    pub file: PathBuf,

    /// Task identifier such as `TASK-3`
    ///
    /// Plain numbers are also accepted and normalized (e.g., "3" → "TASK-3").
    pub task_id: String,
}

/// Parameters for new tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewParams {
    /// Path to the tasks file
    #[serde(default = "default_task_file")]
    pub file: PathBuf,

    /// Task body text
    ///
    /// This becomes the full task content. Can be multi-line.
    /// The task will be created with state "todo" and a unique ID.
    pub content: String,
}

/// Parameters for edit-state tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EditStateParams {
    /// Path to the tasks file
    #[serde(default = "default_task_file")]
    pub file: PathBuf,

    /// Task identifier such as `TASK-3`
    pub task_id: String,

    /// New state value to write into the task
    ///
    /// Common states: "todo", "in-progress", "review", "done", "blocked"
    pub state: String,
}

/// Parameters for edit-content tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EditContentParams {
    /// Path to the tasks file
    #[serde(default = "default_task_file")]
    pub file: PathBuf,

    /// Task identifier such as `TASK-3`
    pub task_id: String,

    /// New full task body that replaces the current content
    pub content: String,
}

/// Parameters for add-content tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddContentParams {
    /// Path to the tasks file
    #[serde(default = "default_task_file")]
    pub file: PathBuf,

    /// Task identifier such as `TASK-3`
    pub task_id: String,

    /// Text to append to the existing task body
    pub content: String,
}

/// Parameters for add-depends-on tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddDependsOnParams {
    /// Path to the tasks file
    #[serde(default = "default_task_file")]
    pub file: PathBuf,

    /// Task identifier such as `TASK-3`
    pub task_id: String,

    /// Identifier of the task this task depends on
    ///
    /// The dependency task must exist. This creates a blocking relationship:
    /// the task cannot be claimed until its dependencies are in "done" state.
    pub depends_on: String,
}

/// Parameters for add-epic tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddEpicParams {
    /// Path to the tasks file
    #[serde(default = "default_task_file")]
    pub file: PathBuf,

    /// Task identifier such as `TASK-3`
    pub task_id: String,

    /// Epic name to add to the task
    ///
    /// Epics are labels for grouping related tasks. A task can belong
    /// to multiple epics.
    pub epic: String,
}

/// Parameters for del-depends-on tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DelDependsOnParams {
    /// Path to the tasks file
    #[serde(default = "default_task_file")]
    pub file: PathBuf,

    /// Task identifier such as `TASK-3`
    pub task_id: String,

    /// Dependency task identifier to remove
    pub depends_on: String,
}

/// Parameters for del-epic tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DelEpicParams {
    /// Path to the tasks file
    #[serde(default = "default_task_file")]
    pub file: PathBuf,

    /// Task identifier such as `TASK-3`
    pub task_id: String,

    /// Epic name to remove from the task
    pub epic: String,
}

/// Parameters for claim tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClaimParams {
    /// Path to the tasks file
    #[serde(default = "default_task_file")]
    pub file: PathBuf,

    /// State to assign to the claimed task
    ///
    /// Typically "in-progress" to indicate work has started.
    pub new_state: String,

    /// Only consider tasks currently in this source state
    ///
    /// Defaults to "todo". Use this to claim tasks from a specific state.
    #[serde(default = "default_claim_state")]
    pub state: String,

    /// If set, only consider tasks that belong to this epic
    ///
    /// Use to claim work from a specific project or feature area.
    pub epic: Option<String>,
}

// ============================================================================
// Handler Implementation
// ============================================================================

/// Handler for minitask MCP tools
#[derive(Clone)]
pub struct MinitaskHandler {
    pub(crate) tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

impl MinitaskHandler {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl MinitaskHandler {
    /// Return tasks from the current task file. Use filters to narrow the result set before doing
    /// follow-up actions such as claim, show, edit, add, or delete
    ///
    /// Search through tasks with optional filtering by state and epic. Returns a list of tasks
    /// matching the criteria. Use `verbose=false` (default) for a quick overview showing just
    /// task IDs and first lines, or `verbose=true` to get complete task details including
    /// dependencies, epics, and full content.
    ///
    /// Returns a JSON object: `{ tasks: [...] }`. Each task includes:
    /// - name: Task ID (e.g., "TASK-3")
    /// - state: Current state (e.g., "todo", "in-progress", "done")
    /// - content: Task description (empty if verbose=false)
    /// - depends_on: List of task IDs this task depends on
    /// - epic: List of epic labels
    ///
    /// Workflow tips:
    /// - Start with list to see available tasks
    /// - Use state filter to focus on specific workflow stages
    /// - Use epic filter to work on specific projects
    /// - Use verbose=true only when you need full details
    /// - Combine filters: state="todo" + epic="feature-x" for targeted work
    #[tool(name = "list")]
    async fn list(&self, params: Parameters<ListParams>) -> String {
        info!(
            "MCP list tool called with file: {:?}, state: {:?}, epic: {:?}, verbose: {}",
            params.0.file, params.0.state, params.0.epic, params.0.verbose
        );

        match execute_list(params.0) {
            Ok(result) => {
                info!("List tool returned {} tasks", result.tasks.len());
                debug!("Tasks: {:?}", result.tasks);
                serde_json::to_string_pretty(&result)
                    .unwrap_or_else(|_| r#"{"tasks":[]}"#.to_string())
            }
            Err(e) => {
                error!("List tool error: {}", e);
                format!(r#"{{"error":"{}"}}"#, e)
            }
        }
    }

    /// Return one task by ID. Use this when you already know the task ID and need the
    /// current full task record before deciding what to change
    ///
    /// Retrieves complete details for a specific task including its current state,
    /// content, dependencies, and epic labels. Use this to:
    /// - Check task status before updating
    /// - Review dependencies before claiming
    /// - Get full context before editing content
    ///
    /// Returns a JSON object: `{ task: {...} }` with complete task details.
    ///
    /// Workflow:
    /// 1. Use list to find tasks
    /// 2. Use show to get full details of a specific task
    /// 3. Use edit/add tools to modify the task
    #[tool(name = "show")]
    async fn show(&self, params: Parameters<ShowParams>) -> String {
        match execute_show(params.0) {
            Ok(result) => {
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
            }
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    /// Create a new task. The provided content becomes the full task body
    ///
    /// Creates a new task with a unique ID (e.g., "TASK-0", "TASK-1", etc.) and
    /// initial state "todo". The task is immediately saved to the file.
    ///
    /// Returns a JSON object: `{ task: {...} }` with the newly created task details
    /// including its assigned ID.
    ///
    /// Use this to:
    /// - Add new work items to track
    /// - Break down larger tasks into subtasks
    /// - Capture ideas or bugs to address later
    ///
    /// After creation, you can:
    /// - Use add-epic to categorize the task
    /// - Use add-depends-on to link it to other tasks
    /// - Use edit-state to move it through your workflow
    #[tool(name = "new")]
    async fn new_task(&self, params: Parameters<NewParams>) -> String {
        match execute_new(params.0) {
            Ok(result) => {
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
            }
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    /// Change the `state` field of an existing task
    ///
    /// Updates a task's state to reflect its current status in your workflow.
    /// Common state transitions:
    /// - "todo" → "in-progress": Start work
    /// - "in-progress" → "review": Ready for review
    /// - "review" → "done": Completed and approved
    /// - Any state → "blocked": Waiting on external dependency
    ///
    /// Returns a JSON object: `{ task: {...} }` with the updated task.
    ///
    /// Use this to:
    /// - Track progress through your workflow
    /// - Signal status changes to team members
    /// - Unblock dependent tasks by marking dependencies as "done"
    ///
    /// Tip: Use claim instead if you want to automatically pick the next
    /// available task and move it to "in-progress".
    #[tool(name = "edit-state")]
    async fn edit_state(&self, params: Parameters<EditStateParams>) -> String {
        match execute_edit_state(params.0) {
            Ok(result) => {
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
            }
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    /// Replace the full text content of an existing task
    ///
    /// Completely replaces a task's content with new text. Use this when:
    /// - The task description needs major revision
    /// - Requirements have changed significantly
    /// - You want to rewrite the task from scratch
    ///
    /// Returns a JSON object: `{ task: {...} }` with the updated task.
    ///
    /// For minor updates or additions, consider using add-content instead
    /// to append to the existing content rather than replacing it entirely.
    #[tool(name = "edit-content")]
    async fn edit_content(&self, params: Parameters<EditContentParams>) -> String {
        match execute_edit_content(params.0) {
            Ok(result) => {
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
            }
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    /// Append additional text to the end of an existing task's content
    ///
    /// Adds new text to the end of a task's existing content without replacing
    /// what's already there. Use this to:
    /// - Add progress notes or updates
    /// - Append new requirements or details
    /// - Record decisions or changes
    /// - Add links or references
    ///
    /// Returns a JSON object: `{ task: {...} }` with the updated task.
    ///
    /// This is safer than edit-content when you want to preserve existing
    /// information while adding new details.
    #[tool(name = "add-content")]
    async fn add_content(&self, params: Parameters<AddContentParams>) -> String {
        match execute_add_content(params.0) {
            Ok(result) => {
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
            }
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    /// Add a dependency so this task records that it depends on another task
    ///
    /// Creates a blocking relationship between tasks. The dependent task cannot
    /// be claimed (via the claim tool) until all its dependencies are in "done" state.
    ///
    /// Returns a JSON object: `{ task: {...} }` with the updated task.
    ///
    /// Use this to:
    /// - Model task prerequisites and ordering
    /// - Prevent work from starting too early
    /// - Track which tasks block others
    ///
    /// Example workflow:
    /// 1. Create tasks for a feature
    /// 2. Use add-depends-on to link them (e.g., "implement" depends on "design")
    /// 3. Use claim to automatically get the next unblocked task
    ///
    /// The dependency task must exist in the same file.
    #[tool(name = "add-depends-on")]
    async fn add_depends_on(&self, params: Parameters<AddDependsOnParams>) -> String {
        match execute_add_depends_on(params.0) {
            Ok(result) => {
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
            }
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    /// Attach an epic label to an existing task
    ///
    /// Adds an epic label to categorize and group related tasks. A task can belong
    /// to multiple epics. Use epics to:
    /// - Group tasks by feature, project, or theme
    /// - Filter tasks by area of work
    /// - Track progress across related tasks
    ///
    /// Returns a JSON object: `{ task: {...} }` with the updated task.
    ///
    /// Common epic patterns:
    /// - Feature names: "user-auth", "payment-system"
    /// - Project phases: "mvp", "beta", "v2"
    /// - Work types: "bug", "feature", "refactor"
    /// - Teams or areas: "frontend", "backend", "docs"
    ///
    /// Use list with epic filter to see all tasks in an epic.
    #[tool(name = "add-epic")]
    async fn add_epic(&self, params: Parameters<AddEpicParams>) -> String {
        match execute_add_epic(params.0) {
            Ok(result) => {
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
            }
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    /// Remove one dependency from an existing task
    ///
    /// Removes a blocking relationship between tasks. Use this when:
    /// - A dependency is no longer needed
    /// - Requirements have changed
    /// - You want to unblock a task
    ///
    /// Returns a JSON object: `{ task: {...} }` with the updated task.
    ///
    /// This does not delete the dependency task itself, only removes the
    /// relationship.
    #[tool(name = "del-depends-on")]
    async fn del_depends_on(&self, params: Parameters<DelDependsOnParams>) -> String {
        match execute_del_depends_on(params.0) {
            Ok(result) => {
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
            }
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    /// Remove one epic label from an existing task
    ///
    /// Removes an epic label from a task. Use this when:
    /// - A task no longer belongs to that epic
    /// - You're reorganizing your epic structure
    /// - The epic is complete or cancelled
    ///
    /// Returns a JSON object: `{ task: {...} }` with the updated task.
    ///
    /// The task remains in the file; only the epic label is removed.
    #[tool(name = "del-epic")]
    async fn del_epic(&self, params: Parameters<DelEpicParams>) -> String {
        match execute_del_epic(params.0) {
            Ok(result) => {
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
            }
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    /// Move the next matching task from one state to another and return the claimed task.
    /// Use this to reserve work before editing it further
    ///
    /// Automatically finds and claims the next available task matching your filters.
    /// A task is available if:
    /// - It matches the state filter (default: "todo")
    /// - It matches the epic filter (if specified)
    /// - All its dependencies are in "done" state
    ///
    /// Returns a JSON object: `{ task: {...} }` with the claimed task details.
    ///
    /// Use this to:
    /// - Pick the next task to work on
    /// - Ensure dependencies are satisfied before starting
    /// - Automatically move tasks through your workflow
    ///
    /// Workflow:
    /// 1. Use claim to get the next available task (moves to "in-progress")
    /// 2. Work on the task
    /// 3. Use edit-state to move to "done" when complete
    /// 4. Repeat to claim the next task
    ///
    /// If no tasks are available (all blocked or none match filters), returns
    /// an error. Use list first to check what's available.
    #[tool]
    async fn claim(&self, params: Parameters<ClaimParams>) -> String {
        match execute_claim(params.0) {
            Ok(result) => {
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
            }
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }
}

#[rmcp::tool_handler(router = self.tool_router)]
impl rmcp::ServerHandler for MinitaskHandler {
    fn get_info(&self) -> ServerInfo {
        let metadata = serde_json::json!({
            "description": env!("CARGO_PKG_DESCRIPTION"),
            "authors": env!("CARGO_PKG_AUTHORS"),
            "license": env!("CARGO_PKG_LICENSE"),
            "repository": env!("CARGO_PKG_REPOSITORY"),
        });
        
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ).with_description(&metadata.to_string()))
            .with_instructions(
                "Minitask MCP server provides task management tools for tracking work items.\n\
                Each tool accepts a 'file' parameter (defaults to 'tasks.toml').\n\
                \n\
                Core workflow:\n\
                • list: See available tasks (use filters to narrow results)\n\
                • claim: Automatically pick the next unblocked task\n\
                • show: Get full details of a specific task\n\
                • edit-state: Move tasks through your workflow\n\
                \n\
                Task creation:\n\
                • new: Create a new task with unique ID\n\
                \n\
                Task organization:\n\
                • add-epic / del-epic: Categorize tasks by project or feature\n\
                • add-depends-on / del-depends-on: Model task dependencies\n\
                \n\
                Task updates:\n\
                • edit-content: Replace task description\n\
                • add-content: Append to task description\n\
                \n\
                Recommended flow:\n\
                1. list to see what's available (optionally filter by state/epic)\n\
                2. claim to automatically get the next unblocked task\n\
                3. edit-state to mark progress (in-progress → review → done)\n\
                4. Repeat\n\
                \n\
                Task states: todo, in-progress, review, done, blocked\n\
                Dependencies: Tasks with dependencies can't be claimed until dependencies are done\n\
                Epics: Labels for grouping related tasks (a task can have multiple epics)\n\
                "
            )
    }
}
