use clap::{CommandFactory, Parser};
use minitask::*;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Error> {
    // Check for --long-help before handing off to clap_mcp, because clap_mcp
    // re-parses argv itself and would not know about our custom flag.
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--long-help") {
        let mut cmd = Cli::command();
        print_full_help(&mut cmd);
        std::process::exit(0);
    }

    // Use a default file path for the initial state; clap_mcp will parse
    // --file from argv on every tool call via mcp_pass/command_pass.
    let state = Arc::new(Mutex::new(State {
        cli_out: false,
        file: "tasks.toml".into(),
    }));

    let cli = Cli::parse();

    if cli.gui {
        serve_gui(Arc::clone(&state), cli.file.clone())?;
        return Ok(());
    }

    if cli.mcp {
        serve_mcp(state)?;
        return Ok(());
    }

    cli_pass(cli)?;

    Ok(())
}
