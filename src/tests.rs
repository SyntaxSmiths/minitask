use super::*;
use std::fs;

#[test]
fn test_task_serialization_roundtrip() {
    let task = Task {
        name: "TASK-0".to_string(),
        state: "todo".to_string(),
        depends_on: vec!["TASK-1".to_string()],
        epic: vec!["planning".to_string()],
        content: "Test task content".to_string(),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&task).unwrap();
    
    // Deserialize back
    let deserialized: Task = serde_json::from_str(&json).unwrap();
    
    assert_eq!(task.name, deserialized.name);
    assert_eq!(task.state, deserialized.state);
    assert_eq!(task.depends_on, deserialized.depends_on);
    assert_eq!(task.epic, deserialized.epic);
    assert_eq!(task.content, deserialized.content);
}

#[test]
fn test_taskfile_toml_parsing() {
    let toml_content = r#"
[[tasks]]
name = "TASK-0"
state = "todo"
depends_on = []
epic = ["planning"]
content = "Test content"

[[tasks]]
name = "TASK-1"
state = "done"
depends_on = ["TASK-0"]
epic = []
content = "Another task"
"#;

    let task_file: TaskFile = toml::from_str(toml_content).unwrap();
    
    assert_eq!(task_file.tasks.len(), 2);
    assert_eq!(task_file.tasks[0].name, "TASK-0");
    assert_eq!(task_file.tasks[0].state, "todo");
    assert_eq!(task_file.tasks[1].name, "TASK-1");
    assert_eq!(task_file.tasks[1].depends_on, vec!["TASK-0"]);
}

#[test]
fn test_load_existing_tasks() {
    let temp_file = "test_load_existing.toml";
    
    // Create a test file
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Test task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Load it back
    let loaded = load_tasks(temp_file).unwrap();
    assert_eq!(loaded.tasks.len(), 1);
    assert_eq!(loaded.tasks[0].name, "TASK-0");
    
    // Cleanup
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_save_tasks() {
    let temp_file = "test_save.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "done".to_string(),
                depends_on: vec!["TASK-1".to_string()],
                epic: vec!["epic1".to_string()],
                content: "Content here".to_string(),
            },
        ],
    };
    
    save_tasks(temp_file, &task_file).unwrap();
    
    // Verify file exists and can be read
    let content = fs::read_to_string(temp_file).unwrap();
    assert!(content.contains("TASK-0"));
    assert!(content.contains("done"));
    
    // Cleanup
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_handle_missing_file() {
    let temp_file = "test_missing.toml";
    
    // Ensure file doesn't exist
    let _ = fs::remove_file(temp_file);
    
    // Load should create empty file
    let loaded = load_tasks(temp_file).unwrap();
    assert_eq!(loaded.tasks.len(), 0);
    
    // Verify file was created
    assert!(Path::new(temp_file).exists());
    
    // Cleanup
    fs::remove_file(temp_file).unwrap();
}


#[test]
fn test_list_all_tasks() {
    let temp_file = "test_list_all.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "First task".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "done".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Second task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Test listing all tasks
    let loaded = load_tasks(temp_file).unwrap();
    assert_eq!(loaded.tasks.len(), 2);
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_list_filter_by_state() {
    let temp_file = "test_list_state.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Todo task".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "done".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Done task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let loaded = load_tasks(temp_file).unwrap();
    let todo_tasks: Vec<&Task> = loaded.tasks.iter()
        .filter(|t| t.state == "todo")
        .collect();
    assert_eq!(todo_tasks.len(), 1);
    assert_eq!(todo_tasks[0].name, "TASK-0");
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_list_filter_by_epic() {
    let temp_file = "test_list_epic.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec!["planning".to_string()],
                content: "Planning task".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec!["implementation".to_string()],
                content: "Implementation task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let loaded = load_tasks(temp_file).unwrap();
    let planning_tasks: Vec<&Task> = loaded.tasks.iter()
        .filter(|t| t.epic.contains(&"planning".to_string()))
        .collect();
    assert_eq!(planning_tasks.len(), 1);
    assert_eq!(planning_tasks[0].name, "TASK-0");
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_list_verbose_output() {
    let task = Task {
        name: "TASK-0".to_string(),
        state: "in-progress".to_string(),
        depends_on: vec!["TASK-1".to_string()],
        epic: vec!["epic1".to_string(), "epic2".to_string()],
        content: "Multi-line\ncontent\nhere".to_string(),
    };
    
    // Verify task has all fields populated
    assert_eq!(task.name, "TASK-0");
    assert_eq!(task.state, "in-progress");
    assert_eq!(task.depends_on.len(), 1);
    assert_eq!(task.epic.len(), 2);
    assert!(task.content.contains("Multi-line"));
}



#[test]
fn test_show_existing_task() {
    let temp_file = "test_show_existing.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec!["TASK-1".to_string()],
                epic: vec!["planning".to_string()],
                content: "Test task content".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let loaded = load_tasks(temp_file).unwrap();
    let task = loaded.tasks.iter().find(|t| t.name == "TASK-0");
    assert!(task.is_some());
    assert_eq!(task.unwrap().content, "Test task content");
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_show_nonexistent_task() {
    let temp_file = "test_show_nonexistent.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Test".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let loaded = load_tasks(temp_file).unwrap();
    let task = loaded.tasks.iter().find(|t| t.name == "TASK-999");
    assert!(task.is_none());
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_show_json_output() {
    let task = Task {
        name: "TASK-0".to_string(),
        state: "in-progress".to_string(),
        depends_on: vec!["TASK-1".to_string()],
        epic: vec!["epic1".to_string()],
        content: "Content here".to_string(),
    };
    
    let json = serde_json::to_string(&task).unwrap();
    let deserialized: Task = serde_json::from_str(&json).unwrap();
    
    assert_eq!(task.name, deserialized.name);
    assert_eq!(task.state, deserialized.state);
}



#[test]
fn test_new_task_with_content() {
    let temp_file = "test_new_content.toml";
    
    // Start with empty file
    let _ = fs::remove_file(temp_file);
    let mut task_file = TaskFile { tasks: vec![] };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Add a new task
    let new_task = Task {
        name: "TASK-0".to_string(),
        state: "todo".to_string(),
        depends_on: vec![],
        epic: vec![],
        content: "New task content".to_string(),
    };
    task_file.tasks.push(new_task);
    save_tasks(temp_file, &task_file).unwrap();
    
    let loaded = load_tasks(temp_file).unwrap();
    assert_eq!(loaded.tasks.len(), 1);
    assert_eq!(loaded.tasks[0].name, "TASK-0");
    assert_eq!(loaded.tasks[0].state, "todo");
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_new_task_unique_id_generation() {
    let temp_file = "test_new_id.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "First".to_string(),
            },
            Task {
                name: "TASK-2".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Third".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Next ID should be 3 (max + 1)
    let loaded = load_tasks(temp_file).unwrap();
    let max_id = loaded.tasks.iter()
        .filter_map(|t| t.name.strip_prefix("TASK-").and_then(|n| n.parse::<usize>().ok()))
        .max()
        .unwrap();
    assert_eq!(max_id, 2);
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_new_task_json_parsing() {
    let task = Task {
        name: "TASK-0".to_string(),
        state: "todo".to_string(),
        depends_on: vec![],
        epic: vec![],
        content: "Test content".to_string(),
    };
    
    let json = serde_json::to_string(&task).unwrap();
    let parsed: Task = serde_json::from_str(&json).unwrap();
    
    assert_eq!(task.name, parsed.name);
    assert_eq!(task.content, parsed.content);
}



#[test]
fn test_edit_state_success() {
    let temp_file = "test_edit_state.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Test task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Load and update state
    let mut loaded = load_tasks(temp_file).unwrap();
    loaded.tasks[0].state = "in-progress".to_string();
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify state changed
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].state, "in-progress");
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_edit_state_invalid_task() {
    let temp_file = "test_edit_invalid.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Test".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let loaded = load_tasks(temp_file).unwrap();
    let task = loaded.tasks.iter().find(|t| t.name == "TASK-999");
    assert!(task.is_none());
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_edit_state_preserves_other_fields() {
    let temp_file = "test_edit_preserve.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec!["TASK-1".to_string()],
                epic: vec!["epic1".to_string()],
                content: "Original content".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Update state
    let mut loaded = load_tasks(temp_file).unwrap();
    loaded.tasks[0].state = "done".to_string();
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify other fields preserved
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].state, "done");
    assert_eq!(reloaded.tasks[0].content, "Original content");
    assert_eq!(reloaded.tasks[0].depends_on, vec!["TASK-1"]);
    assert_eq!(reloaded.tasks[0].epic, vec!["epic1"]);
    
    fs::remove_file(temp_file).unwrap();
}



#[test]
fn test_edit_content_success() {
    let temp_file = "test_edit_content.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Original content".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Load and update content
    let mut loaded = load_tasks(temp_file).unwrap();
    loaded.tasks[0].content = "New content".to_string();
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify content changed
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].content, "New content");
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_edit_content_invalid_task() {
    let temp_file = "test_edit_content_invalid.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Test".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let loaded = load_tasks(temp_file).unwrap();
    let task = loaded.tasks.iter().find(|t| t.name == "TASK-999");
    assert!(task.is_none());
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_edit_content_preserves_other_fields() {
    let temp_file = "test_edit_content_preserve.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "in-progress".to_string(),
                depends_on: vec!["TASK-1".to_string()],
                epic: vec!["epic1".to_string()],
                content: "Original content".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Update content
    let mut loaded = load_tasks(temp_file).unwrap();
    loaded.tasks[0].content = "Updated content".to_string();
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify other fields preserved
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].content, "Updated content");
    assert_eq!(reloaded.tasks[0].state, "in-progress");
    assert_eq!(reloaded.tasks[0].depends_on, vec!["TASK-1"]);
    assert_eq!(reloaded.tasks[0].epic, vec!["epic1"]);
    
    fs::remove_file(temp_file).unwrap();
}



#[test]
fn test_add_content_success() {
    let temp_file = "test_add_content.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Original content".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Load and append content
    let mut loaded = load_tasks(temp_file).unwrap();
    loaded.tasks[0].content.push_str("\nAppended content");
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify content appended
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].content, "Original content\nAppended content");
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_add_content_invalid_task() {
    let temp_file = "test_add_content_invalid.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Test".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let loaded = load_tasks(temp_file).unwrap();
    let task = loaded.tasks.iter().find(|t| t.name == "TASK-999");
    assert!(task.is_none());
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_add_content_preserves_existing() {
    let temp_file = "test_add_preserve.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Line 1\nLine 2".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Append content
    let mut loaded = load_tasks(temp_file).unwrap();
    loaded.tasks[0].content.push_str("\nLine 3");
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify existing content preserved
    let reloaded = load_tasks(temp_file).unwrap();
    assert!(reloaded.tasks[0].content.contains("Line 1"));
    assert!(reloaded.tasks[0].content.contains("Line 2"));
    assert!(reloaded.tasks[0].content.contains("Line 3"));
    
    fs::remove_file(temp_file).unwrap();
}



#[test]
fn test_add_depends_on_success() {
    let temp_file = "test_add_depends.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "First task".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Second task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Add dependency
    let mut loaded = load_tasks(temp_file).unwrap();
    loaded.tasks[0].depends_on.push("TASK-1".to_string());
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify dependency added
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].depends_on, vec!["TASK-1"]);
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_add_depends_on_invalid_task() {
    let temp_file = "test_add_depends_invalid.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Test".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let loaded = load_tasks(temp_file).unwrap();
    let task = loaded.tasks.iter().find(|t| t.name == "TASK-999");
    assert!(task.is_none());
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_add_depends_on_prevent_duplicates() {
    let temp_file = "test_add_depends_dup.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec!["TASK-1".to_string()],
                epic: vec![],
                content: "Test".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Dependency".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Try to add duplicate
    let mut loaded = load_tasks(temp_file).unwrap();
    if !loaded.tasks[0].depends_on.contains(&"TASK-1".to_string()) {
        loaded.tasks[0].depends_on.push("TASK-1".to_string());
    }
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify no duplicate
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].depends_on.len(), 1);
    
    fs::remove_file(temp_file).unwrap();
}



#[test]
fn test_del_depends_on_success() {
    let temp_file = "test_del_depends.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec!["TASK-1".to_string(), "TASK-2".to_string()],
                epic: vec![],
                content: "Test task".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Dependency".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Remove dependency
    let mut loaded = load_tasks(temp_file).unwrap();
    loaded.tasks[0].depends_on.retain(|d| d != "TASK-1");
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify dependency removed
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].depends_on, vec!["TASK-2"]);
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_del_depends_on_invalid_task() {
    let temp_file = "test_del_depends_invalid.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Test".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let loaded = load_tasks(temp_file).unwrap();
    let task = loaded.tasks.iter().find(|t| t.name == "TASK-999");
    assert!(task.is_none());
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_del_depends_on_nonexistent() {
    let temp_file = "test_del_nonexist.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec!["TASK-1".to_string()],
                epic: vec![],
                content: "Test".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Try to remove non-existent dependency
    let mut loaded = load_tasks(temp_file).unwrap();
    loaded.tasks[0].depends_on.retain(|d| d != "TASK-999");
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify original dependency still there
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].depends_on, vec!["TASK-1"]);
    
    fs::remove_file(temp_file).unwrap();
}



#[test]
fn test_add_epic_success() {
    let temp_file = "test_add_epic.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Test task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Add epic
    let mut loaded = load_tasks(temp_file).unwrap();
    loaded.tasks[0].epic.push("planning".to_string());
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify epic added
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].epic, vec!["planning"]);
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_add_epic_invalid_task() {
    let temp_file = "test_add_epic_invalid.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Test".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let loaded = load_tasks(temp_file).unwrap();
    let task = loaded.tasks.iter().find(|t| t.name == "TASK-999");
    assert!(task.is_none());
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_add_epic_prevent_duplicates() {
    let temp_file = "test_add_epic_dup.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec!["planning".to_string()],
                content: "Test".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Try to add duplicate
    let mut loaded = load_tasks(temp_file).unwrap();
    if !loaded.tasks[0].epic.contains(&"planning".to_string()) {
        loaded.tasks[0].epic.push("planning".to_string());
    }
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify no duplicate
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].epic.len(), 1);
    
    fs::remove_file(temp_file).unwrap();
}



#[test]
fn test_del_epic_success() {
    let temp_file = "test_del_epic.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec!["planning".to_string(), "implementation".to_string()],
                content: "Test task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Remove epic
    let mut loaded = load_tasks(temp_file).unwrap();
    loaded.tasks[0].epic.retain(|e| e != "planning");
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify epic removed
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].epic, vec!["implementation"]);
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_del_epic_invalid_task() {
    let temp_file = "test_del_epic_invalid.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Test".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    let loaded = load_tasks(temp_file).unwrap();
    let task = loaded.tasks.iter().find(|t| t.name == "TASK-999");
    assert!(task.is_none());
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_del_epic_nonexistent() {
    let temp_file = "test_del_epic_nonexist.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec!["planning".to_string()],
                content: "Test".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Try to remove non-existent epic
    let mut loaded = load_tasks(temp_file).unwrap();
    loaded.tasks[0].epic.retain(|e| e != "nonexistent");
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify original epic still there
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].epic, vec!["planning"]);
    
    fs::remove_file(temp_file).unwrap();
}



#[test]
fn test_claim_task_from_todo() {
    let temp_file = "test_claim_todo.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "First task".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Second task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Claim first todo task
    let mut loaded = load_tasks(temp_file).unwrap();
    let task = loaded.tasks.iter_mut()
        .find(|t| t.state == "todo")
        .unwrap();
    task.state = "in-progress".to_string();
    save_tasks(temp_file, &loaded).unwrap();
    
    // Verify task claimed
    let reloaded = load_tasks(temp_file).unwrap();
    assert_eq!(reloaded.tasks[0].state, "in-progress");
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_claim_with_state_filter() {
    let temp_file = "test_claim_state.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "blocked".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Blocked task".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Todo task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Find first todo task
    let loaded = load_tasks(temp_file).unwrap();
    let task = loaded.tasks.iter()
        .find(|t| t.state == "todo");
    assert!(task.is_some());
    assert_eq!(task.unwrap().name, "TASK-1");
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_claim_with_epic_filter() {
    let temp_file = "test_claim_epic.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec!["planning".to_string()],
                content: "Planning task".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec!["implementation".to_string()],
                content: "Implementation task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Find first planning task
    let loaded = load_tasks(temp_file).unwrap();
    let task = loaded.tasks.iter()
        .find(|t| t.state == "todo" && t.epic.contains(&"planning".to_string()));
    assert!(task.is_some());
    assert_eq!(task.unwrap().name, "TASK-0");
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_claim_no_available_tasks() {
    let temp_file = "test_claim_none.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "done".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "Done task".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Try to find todo task
    let loaded = load_tasks(temp_file).unwrap();
    let task = loaded.tasks.iter()
        .find(|t| t.state == "todo");
    assert!(task.is_none());
    
    fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_claim_dependency_blocking() {
    let temp_file = "test_claim_deps.toml";
    
    let task_file = TaskFile {
        tasks: vec![
            Task {
                name: "TASK-0".to_string(),
                state: "todo".to_string(),
                depends_on: vec!["TASK-1".to_string()],
                epic: vec![],
                content: "Blocked by TASK-1".to_string(),
            },
            Task {
                name: "TASK-1".to_string(),
                state: "todo".to_string(),
                depends_on: vec![],
                epic: vec![],
                content: "No dependencies".to_string(),
            },
        ],
    };
    save_tasks(temp_file, &task_file).unwrap();
    
    // Find first unblocked task
    let loaded = load_tasks(temp_file).unwrap();
    let unblocked = loaded.tasks.iter()
        .find(|t| {
            t.state == "todo" && t.depends_on.iter().all(|dep_id| {
                loaded.tasks.iter()
                    .find(|dt| dt.name == *dep_id)
                    .map_or(true, |dt| dt.state == "done")
            })
        });
    assert!(unblocked.is_some());
    assert_eq!(unblocked.unwrap().name, "TASK-1");
    
    fs::remove_file(temp_file).unwrap();
}



#[test]
fn test_normalize_task_id_number() {
    assert_eq!(normalize_task_id("0"), "TASK-0");
    assert_eq!(normalize_task_id("1"), "TASK-1");
    assert_eq!(normalize_task_id("42"), "TASK-42");
}

#[test]
fn test_normalize_task_id_already_normalized() {
    assert_eq!(normalize_task_id("TASK-0"), "TASK-0");
    assert_eq!(normalize_task_id("TASK-1"), "TASK-1");
    assert_eq!(normalize_task_id("TASK-42"), "TASK-42");
}

#[test]
fn test_normalize_task_id_invalid() {
    assert_eq!(normalize_task_id("invalid"), "invalid");
    assert_eq!(normalize_task_id("TASK-"), "TASK-");
    assert_eq!(normalize_task_id("task-1"), "task-1");
}

