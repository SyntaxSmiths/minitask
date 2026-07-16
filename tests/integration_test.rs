//! Integration tests for minitask
//! These tests verify end-to-end functionality through the public API

use minitask::{Commands, TaskFile, Task};

#[test]
fn test_task_serialization() {
    let task = Task::new(
        "TASK-0",
        "todo",
        vec!["TASK-1"],
        vec!["epic1"],
        "Test content",
    );
    
    let json = serde_json::to_string(&task).unwrap();
    let deserialized: Task = serde_json::from_str(&json).unwrap();
    
    assert_eq!(task.name, deserialized.name);
    assert_eq!(task.state, deserialized.state);
    assert_eq!(task.depends_on, deserialized.depends_on);
    assert_eq!(task.epic, deserialized.epic);
    assert_eq!(task.content, deserialized.content);
}

#[test]
fn test_taskfile_toml_roundtrip() {
    let task_file = TaskFile {
        tasks: vec![
            Task::new("TASK-0", "todo", vec![], vec!["planning"], "First task"),
            Task::new("TASK-1", "done", vec!["TASK-0"], vec![], "Second task"),
        ],
    };
    
    let toml = toml::to_string(&task_file).unwrap();
    let parsed: TaskFile = toml::from_str(&toml).unwrap();
    
    assert_eq!(parsed.tasks.len(), 2);
    assert_eq!(parsed.tasks[0].name, "TASK-0");
    assert_eq!(parsed.tasks[1].depends_on, vec!["TASK-0"]);
}

#[test]
fn test_commands_serialization() {
    let cmd = Commands::List {
        state: Some("todo".to_string()),
        epic: None,
        verbose: false,
    };
    
    let json = serde_json::to_string(&cmd).unwrap();
    let deserialized: Commands = serde_json::from_str(&json).unwrap();
    
    match deserialized {
        Commands::List { state, epic, verbose } => {
            assert_eq!(state, Some("todo".to_string()));
            assert_eq!(epic, None);
            assert_eq!(verbose, false);
        }
        _ => panic!("Wrong command variant"),
    }
}
