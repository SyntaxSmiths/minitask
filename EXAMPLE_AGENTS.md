# mandatory workflow
- use **minitask** cli tool for ALL tasks during programming
- NEVER use the tool update_todo_list. Always use minitask
- use minitask for tasks with only one step
- Do NOT use markdown TODOs, task lists, or other tracking methods
- tasks content always follow the 50/72 format: 
   - 50 characters heading line
   - blank line
   - block of explanation maximum 72 characters wide and maximum 10 lines long
   - blank line
   - list of tests for for this task
   - blank line
   - criteria to finish the task, at least: all tests pass && code documented, you can add more criteria
- the task content should be formulated so it can also be used as a commit message
- a task is only done if all exit criteria is meet
- a task is never done if the project cannot be compiled
- tasks states are: todo -> in-progress -> done
- if a task is in-progress and is blocked:
   - create a new task with blocking work
   - create a dependency of blocked-task to new-task
   - move the blocked-task back to todo
   - move new-task in todo
- the last ask is always push gained knowledge to mempalace
- Link discovered work with dependencies
- Do NOT use external issue trackers
- Do NOT duplicate tracking systems

1. **Check ready work**: `minitask list --state todo` shows unblocked tasks
2. **Claim your task atomically**: `minitask claim in-progress`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked task:
   - `minitask new "Found bug: Details about what was found"`
   - `minitask add depends-on <new-id> <parent-id>`
5. **Complete**: `minitask edit state <id> done`

# commands

## Minitask Commands

| Command | Description |
|---------|-------------|
| `minitask list` | List all tasks |
| `minitask list --state [state]` | List tasks in specific state |
| `minitask list --epic [epic]` | List tasks in specific epic |
| `minitask list --verbose` | List tasks with full details |
| `minitask show [task-id]` | Show complete task details |
| `minitask new "content"` | Create new task |
| `minitask new -` | Create new task with content from stdin |
| `minitask edit state [task-id] [state]` | Change task state |
| `minitask edit content [task-id] "content"` | Replace task content |
| `minitask add content [task-id] "content"` | Append to task content |
| `minitask add depends-on [task-id] [other-id]` | Add task dependency |
| `minitask del depends-on [task-id] [other-id]` | Remove task dependency |
| `minitask add epic [task-id] [epic-id]` | Add task to epic |
| `minitask del epic [task-id] [epic-id]` | Remove task from epic |
| `minitask claim [new-state]` | Claim next task from todo (default) and move to new state |
| `minitask claim [new-state] --from-state [state]` | Claim next task from specified state |
| `minitask claim [new-state] --epic [epic]` | Claim next task from specific epic |
| `minitask claim [new-state] -f [state] -e [epic]` | Claim with custom state and epic filters |