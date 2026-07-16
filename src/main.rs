use clap::{CommandFactory, Parser};
use minitask::*;
use log::{debug, info};

/// Task management CLI tool
#[derive(Parser, Debug)]
#[command(name = "minitask", about = "A simple task management tool", long_about = None, arg_required_else_help = true)]
struct Cli {
    /// Start the GTK GUI via gjs.
    #[arg(long, global = true, action = clap::ArgAction::SetTrue)]
    gui: bool,

    /// Start the MCP server over stdio.
    #[arg(long, global = true, action = clap::ArgAction::SetTrue)]
    mcp: bool,

    /// Path to the tasks file
    #[arg(long, global = true, default_value = "tasks.toml")]
    file: std::path::PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

fn print_full_help(cmd: &mut clap::Command) {
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

    // Header so it's obvious which command we're printing
    println!("========== {} ==========\n", full_path);

    cmd.print_long_help().unwrap();
    println!("\n");

    // Recurse into subcommands
    for sub in cmd.get_subcommands_mut() {
        print_full_help_inner(sub, &full_path);
    }
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

fn execute_command(tasks_file: &mut TaskFile, command: Commands) -> Result<(), Error> {
    match command {
        Commands::LongHelp => unreachable!("LongHelp should be handled in main"),
        Commands::List {
            state,
            epic,
            verbose,
        } => {
            let filtered = tasks_file.handle_list(state.as_deref(), epic.as_deref());
            if verbose {
                for task in &filtered {
                    print_task_verbose(task);
                }
            } else {
                for task in &filtered {
                    let first_line = task.content.lines().next().unwrap_or("");
                    println!("{}: {}", task.name, first_line);
                }
            }
            Ok(())
        }
        Commands::Show { task_id } => {
            let task = tasks_file.handle_show(&task_id)?;
            print_task_verbose(task);
            Ok(())
        }
        Commands::New { content } => {
            let task = tasks_file.handle_new(&content)?;
            println!("Created {}", task.name);
            Ok(())
        }
        Commands::EditState { task_id, state } => {
            tasks_file.handle_edit_state(&task_id, &state)?;
            println!("Updated {} state to {}", normalize_task_id(&task_id), state);
            Ok(())
        }
        Commands::EditContent { task_id, content } => {
            tasks_file.handle_edit_content(&task_id, &content)?;
            println!("Updated {} content", normalize_task_id(&task_id));
            Ok(())
        }
        Commands::AddContent { task_id, content } => {
            tasks_file.handle_add_content(&task_id, &content)?;
            println!("Appended content to {}", normalize_task_id(&task_id));
            Ok(())
        }
        Commands::AddDependsOn {
            task_id,
            depends_on,
        } => {
            tasks_file.handle_add_depends_on(&task_id, &depends_on)?;
            println!("Added dependency {} to {}", normalize_task_id(&depends_on), normalize_task_id(&task_id));
            Ok(())
        }
        Commands::AddEpic { task_id, epic } => {
            tasks_file.handle_add_epic(&task_id, &epic)?;
            println!("Added epic {} to {}", epic, normalize_task_id(&task_id));
            Ok(())
        }
        Commands::DelDependsOn {
            task_id,
            depends_on,
        } => {
            tasks_file.handle_del_depends_on(&task_id, &depends_on)?;
            println!("Removed dependency {} from {}", depends_on, normalize_task_id(&task_id));
            Ok(())
        }
        Commands::DelEpic { task_id, epic } => {
            tasks_file.handle_del_epic(&task_id, &epic)?;
            println!("Removed epic {} from {}", epic, normalize_task_id(&task_id));
            Ok(())
        }
        Commands::Claim {
            new_state,
            state,
            epic,
        } => {
            let task = tasks_file.handle_claim(&new_state, &state, epic.as_deref())?;
            println!("Claimed {} and moved to {}", task.name, new_state);
            Ok(())
        }
    }
}

fn main() -> Result<(), Error> {
    // Initialize logger - use RUST_LOG env var to control level
    // Example: RUST_LOG=debug minitask --gui
    env_logger::init();
    
    let cli = Cli::parse();

    info!("Starting minitask");
    debug!("CLI args: {:?}", cli);

    if cli.gui {
        info!("Starting GUI mode with file: {:?}", cli.file);
        serve_gui(cli.file)?;
        return Ok(());
    }

    if cli.mcp {
        info!("Starting MCP mode");
        serve_mcp()?;
        return Ok(());
    }

    let command = cli.command.ok_or_else(|| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No command provided",
        ))
    })?;

    // Handle long-help as a special command
    if matches!(command, Commands::LongHelp) {
        let mut cmd = Cli::command();
        print_full_help(&mut cmd);
        return Ok(());
    }

    // Execute CLI command with file locking
    use file_lock::{FileLock, FileOptions};
    let options = FileOptions::new().write(true).read(true).create(true);
    let mut file_lock = FileLock::lock(&cli.file, false, options)?;
    let mut tasks_file = TaskFile::load(&mut file_lock.file)?;

    let result = execute_command(&mut tasks_file, command);
    
    if result.is_ok() {
        tasks_file.save(&mut file_lock.file)?;
    }

    result
}
