# minitask

A simple, efficient task management CLI tool written in Rust.

## Features

- **Simple TOML storage** - Tasks stored in human-readable `tasks.toml`
- **Dependency tracking** - Tasks can depend on other tasks
- **Epic organization** - Group tasks into epics
- **State management** - Track task progress (todo, in-progress, done, etc.)
- **Smart claiming** - Automatically claim next available task with dependency checking
- **JSON support** - Input/output in JSON format for scripting
- **Filtering** - Filter tasks by state or epic
- **Verbose mode** - Detailed task information when needed

## Installation

```bash
cargo build --release
sudo cp target/release/minitask /usr/local/bin/
```

## Quick Start

```bash
# Create a new task
minitask new "Implement user authentication"

# List all tasks
minitask list

# Claim next available task
minitask claim in-progress

# Mark task as done
minitask edit state TASK-0 done
```

## Commands

### `list` - List tasks

```bash
# List all tasks
minitask list

# Filter by state
minitask list --state todo

# Filter by epic
minitask list --epic backend

# Show verbose output
minitask list --verbose
```

### `show` - Show task details

```bash
# Show task summary
minitask show TASK-0

# Show full details
minitask show TASK-0 --verbose
```

### `new` - Create new task

```bash
# Create task with content
minitask new "Task description"

# Create task from stdin
echo "Task from stdin" | minitask new -

# Create with JSON output
minitask new "New task" --json-out
```

### `edit` - Edit task properties

```bash
# Change task state
minitask edit state TASK-0 in-progress

# Replace task content
minitask edit content TASK-0 "Updated description"
```

### `add` - Add to task properties

```bash
# Append to task content
minitask add content TASK-0 "\nAdditional notes"

# Add dependency
minitask add depends-on TASK-0 TASK-1

# Add to epic
minitask add epic TASK-0 backend
```

### `del` - Remove from task properties

```bash
# Remove dependency
minitask del depends-on TASK-0 TASK-1

# Remove from epic
minitask del epic TASK-0 backend
```

### `claim` - Claim next available task

```bash
# Claim next todo task
minitask claim in-progress

# Claim from specific state
minitask claim in-progress --state blocked

# Claim from specific epic
minitask claim in-progress --epic backend

# Combine filters
minitask claim in-progress --state todo --epic frontend
```

The `claim` command automatically:
- Finds the first task matching filters
- Checks that all dependencies are satisfied (in "done" state)
- Moves the task to the specified state
- Skips tasks with blocking dependencies

## Global Options

- `--file <PATH>` - Use custom task file (default: `tasks.toml`)
- `--json-out` - Output results as JSON
- `--json-in` - Accept input as JSON from stdin

## Task File Format

Tasks are stored in TOML format:

```toml
[[tasks]]
name = "TASK-0"
state = "todo"
depends_on = ["TASK-1"]
epic = ["backend", "api"]
content = "Implement user authentication\n\nRequirements:\n- JWT tokens\n- Password hashing"

[[tasks]]
name = "TASK-1"
state = "done"
depends_on = []
epic = ["backend"]
content = "Set up database schema"
```

## Examples

### Basic Workflow

```bash
# Create tasks
minitask new "Design API endpoints"
minitask new "Implement authentication"
minitask new "Write tests"

# Add dependencies
minitask add depends-on TASK-1 TASK-0
minitask add depends-on TASK-2 TASK-1

# Organize into epic
minitask add epic TASK-0 api-development
minitask add epic TASK-1 api-development
minitask add epic TASK-2 api-development

# Work on tasks
minitask claim in-progress              # Claims TASK-0 (no dependencies)
minitask edit state TASK-0 done
minitask claim in-progress              # Claims TASK-1 (TASK-0 is done)
```

### JSON Integration

```bash
# Get task as JSON
minitask show TASK-0 --json-out | jq .

# Create task from JSON
echo '{"content": "New task"}' | minitask new - --json-in --json-out

# List tasks as JSON for processing
minitask list --json-out | jq '.[] | select(.state == "todo")'
```

### Custom Task File

```bash
# Use project-specific task file
minitask --file project-tasks.toml list
minitask --file project-tasks.toml new "Project task"
```

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests
./integration_tests.sh
```

## Task States

Common task states (customizable):
- `todo` - Not started
- `in-progress` - Currently being worked on
- `blocked` - Waiting on dependencies or external factors
- `done` - Completed
- `cancelled` - No longer needed

## License

MIT

## Contributing

Contributions welcome! Please ensure:
- All tests pass (`cargo test`)
- Integration tests pass (`./integration_tests.sh`)
- Code follows Rust conventions (`cargo fmt`, `cargo clippy`)
