//! Tool execution functions for MCP handlers

use crate::{normalize_task_id, Error, Task, TaskFile};
use file_lock::{FileLock, FileOptions};
use serde::Serialize;

pub use super::handler::{
    AddContentParams, AddDependsOnParams, AddEpicParams, ClaimParams, DelDependsOnParams,
    DelEpicParams, EditContentParams, EditStateParams, ListParams, NewParams, ShowParams,
};

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct TaskListResult {
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskResult {
    pub task: Task,
}

// ============================================================================
// Tool Execution Functions
// ============================================================================

pub fn execute_list(params: ListParams) -> Result<TaskListResult, Error> {
    let options = FileOptions::new().write(false).read(true);
    let mut file_lock = FileLock::lock(&params.file, false, options)?;
    let task_file = TaskFile::load(&mut file_lock.file)?;

    // Use TaskFile method
    let filtered = task_file.handle_list(params.state.as_deref(), params.epic.as_deref());
    
    let mut tasks: Vec<Task> = filtered.into_iter().cloned().collect();

    // If not verbose, clear content to reduce response size
    if !params.verbose {
        tasks.iter_mut().for_each(|task| {
            task.content = String::new();
        });
    }

    Ok(TaskListResult { tasks })
}

pub fn execute_show(params: ShowParams) -> Result<TaskResult, Error> {
    let options = FileOptions::new().write(false).read(true);
    let mut file_lock = FileLock::lock(&params.file, false, options)?;
    let task_file = TaskFile::load(&mut file_lock.file)?;

    // Use TaskFile method
    let task = task_file.handle_show(&params.task_id)?;
    
    Ok(TaskResult {
        task: task.clone(),
    })
}

pub fn execute_new(params: NewParams) -> Result<TaskResult, Error> {
    let options = FileOptions::new().write(true).read(true).create(true);
    let mut file_lock = FileLock::lock(&params.file, false, options)?;
    let mut task_file = TaskFile::load(&mut file_lock.file)?;

    // Use TaskFile method
    let task = task_file.handle_new(&params.content)?;
    
    task_file.save(&mut file_lock.file)?;

    Ok(TaskResult { task })
}

pub fn execute_edit_state(params: EditStateParams) -> Result<TaskResult, Error> {
    let options = FileOptions::new().write(true).read(true);
    let mut file_lock = FileLock::lock(&params.file, false, options)?;
    let mut task_file = TaskFile::load(&mut file_lock.file)?;

    // Use TaskFile method
    task_file.handle_edit_state(&params.task_id, &params.state)?;

    // Get the updated task
    let task_id = normalize_task_id(&params.task_id);
    let task = task_file.select(&task_id)?.clone();

    task_file.save(&mut file_lock.file)?;

    Ok(TaskResult { task })
}

pub fn execute_edit_content(params: EditContentParams) -> Result<TaskResult, Error> {
    let options = FileOptions::new().write(true).read(true);
    let mut file_lock = FileLock::lock(&params.file, false, options)?;
    let mut task_file = TaskFile::load(&mut file_lock.file)?;

    // Use TaskFile method
    task_file.handle_edit_content(&params.task_id, &params.content)?;

    // Get the updated task
    let task_id = normalize_task_id(&params.task_id);
    let task = task_file.select(&task_id)?.clone();

    task_file.save(&mut file_lock.file)?;

    Ok(TaskResult { task })
}

pub fn execute_add_content(params: AddContentParams) -> Result<TaskResult, Error> {
    let options = FileOptions::new().write(true).read(true);
    let mut file_lock = FileLock::lock(&params.file, false, options)?;
    let mut task_file = TaskFile::load(&mut file_lock.file)?;

    // Use TaskFile method
    task_file.handle_add_content(&params.task_id, &params.content)?;

    // Get the updated task
    let task_id = normalize_task_id(&params.task_id);
    let task = task_file.select(&task_id)?.clone();

    task_file.save(&mut file_lock.file)?;

    Ok(TaskResult { task })
}

pub fn execute_add_depends_on(params: AddDependsOnParams) -> Result<TaskResult, Error> {
    let options = FileOptions::new().write(true).read(true);
    let mut file_lock = FileLock::lock(&params.file, false, options)?;
    let mut task_file = TaskFile::load(&mut file_lock.file)?;

    // Use TaskFile method
    task_file.handle_add_depends_on(&params.task_id, &params.depends_on)?;

    // Get the updated task
    let task_id = normalize_task_id(&params.task_id);
    let task = task_file.select(&task_id)?.clone();

    task_file.save(&mut file_lock.file)?;

    Ok(TaskResult { task })
}

pub fn execute_add_epic(params: AddEpicParams) -> Result<TaskResult, Error> {
    let options = FileOptions::new().write(true).read(true);
    let mut file_lock = FileLock::lock(&params.file, false, options)?;
    let mut task_file = TaskFile::load(&mut file_lock.file)?;

    // Use TaskFile method
    task_file.handle_add_epic(&params.task_id, &params.epic)?;

    // Get the updated task
    let task_id = normalize_task_id(&params.task_id);
    let task = task_file.select(&task_id)?.clone();

    task_file.save(&mut file_lock.file)?;

    Ok(TaskResult { task })
}

pub fn execute_del_depends_on(params: DelDependsOnParams) -> Result<TaskResult, Error> {
    let options = FileOptions::new().write(true).read(true);
    let mut file_lock = FileLock::lock(&params.file, false, options)?;
    let mut task_file = TaskFile::load(&mut file_lock.file)?;

    // Use TaskFile method
    task_file.handle_del_depends_on(&params.task_id, &params.depends_on)?;

    // Get the updated task
    let task_id = normalize_task_id(&params.task_id);
    let task = task_file.select(&task_id)?.clone();

    task_file.save(&mut file_lock.file)?;

    Ok(TaskResult { task })
}

pub fn execute_del_epic(params: DelEpicParams) -> Result<TaskResult, Error> {
    let options = FileOptions::new().write(true).read(true);
    let mut file_lock = FileLock::lock(&params.file, false, options)?;
    let mut task_file = TaskFile::load(&mut file_lock.file)?;

    // Use TaskFile method
    task_file.handle_del_epic(&params.task_id, &params.epic)?;

    // Get the updated task
    let task_id = normalize_task_id(&params.task_id);
    let task = task_file.select(&task_id)?.clone();

    task_file.save(&mut file_lock.file)?;

    Ok(TaskResult { task })
}

pub fn execute_claim(params: ClaimParams) -> Result<TaskResult, Error> {
    let options = FileOptions::new().write(true).read(true);
    let mut file_lock = FileLock::lock(&params.file, false, options)?;
    let mut task_file = TaskFile::load(&mut file_lock.file)?;

    // Use TaskFile method
    let task = task_file.handle_claim(&params.new_state, &params.state, params.epic.as_deref())?;

    let result = task.clone();
    
    task_file.save(&mut file_lock.file)?;

    Ok(TaskResult { task: result })
}
