# minitask ![release](https://github.com/SyntaxSmiths/minitask/actions/workflows/release.yml/badge.svg)

A simple, efficient task management CLI tool written in Rust.

## Overview

minitask is a lightweight command-line tool for managing tasks with dependencies, epics, and state tracking. Tasks are stored in a human-readable TOML file.

## Use for AI Agent Control

minitask is particularly effective for AI agent task management with atomic operations, automatic dependency blocking, and structured output. See [MINITASK.md](MINITASK.md#use-for-ai-agent-control) for benefits and [EXAMPLE_AGENTS.md](EXAMPLE_AGENTS.md) for integration examples.

## Quick Start

```bash
# Build and install
cargo build --release
sudo cp target/release/minitask /usr/local/bin/

# Create and manage tasks
minitask new "Implement feature X"
minitask list
minitask claim in-progress
minitask edit-state TASK-0 done
```

## Key Features

- Simple TOML storage format
- Dependency tracking with automatic blocking
- Epic-based organization
- Smart task claiming with dependency resolution
- Filtering by state and epic

## Documentation

For complete command reference, examples, and detailed usage, see [MINITASK.md](MINITASK.md).

## GUI Rendering Note

The GTK task list intentionally remains on `Gtk.ListBox` rather than a GTK 4
model/view stack. The current GUI reloads the visible filtered task set on
explicit refreshes and file-change events, preserves ordering by rebuilding the
same rows in reverse task order, and wires per-row state buttons directly to the
matching task ID. For the expected task counts in this tool, that keeps the
implementation smaller and easier to reason about than introducing
`Gio.ListStore` and `Gtk.ListView`.

## License

MIT
