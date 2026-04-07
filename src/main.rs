use std::path::PathBuf;
use std::process;

use blinc_layout_inspector::server::client;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "blinc-debug")]
#[command(about = "Inspect running Blinc applications via IPC")]
struct Cli {
    /// Path to the Unix socket (auto-detects if only one server is running)
    #[arg(short, long)]
    socket: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Print the layout tree
    Dump,
    /// Print the app state
    State,
    /// Check if the app is running
    Ping,
    /// Save a screenshot to a JPEG file
    Screenshot {
        /// Output file path
        output: PathBuf,
    },
    /// List running debug servers
    List,
}

fn main() {
    let cli = Cli::parse();

    if matches!(cli.command, Some(Commands::List)) {
        list_servers();
        return;
    }

    let socket = match cli.socket.or_else(resolve_socket) {
        Some(s) => s,
        None => {
            eprintln!("No blinc debug server found. Is the app running?");
            process::exit(1);
        }
    };

    let command = cli.command.unwrap_or(Commands::State);
    if let Err(e) = run_command(&socket, command) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn resolve_socket() -> Option<PathBuf> {
    let servers = client::find_servers();
    match servers.len() {
        1 => Some(servers.into_iter().next().unwrap()),
        0 => None,
        _ => {
            eprintln!("Multiple servers found, specify --socket:");
            for s in &servers {
                eprintln!("  {}", s.display());
            }
            process::exit(1);
        }
    }
}

fn list_servers() {
    let servers = client::find_servers();
    if servers.is_empty() {
        println!("No blinc debug servers running");
    } else {
        for s in servers {
            println!("{}", s.display());
        }
    }
}

fn run_command(socket: &PathBuf, command: Commands) -> Result<(), String> {
    match command {
        Commands::Dump => {
            let layout = client::dump(socket).map_err(|e| e.to_string())?;
            print!("{layout}");
        }
        Commands::State => {
            let state = client::state(socket).map_err(|e| e.to_string())?;
            print!("{state}");
        }
        Commands::Ping => {
            client::ping(socket).map_err(|e| e.to_string())?;
            println!("pong");
        }
        Commands::Screenshot { output } => {
            client::screenshot_to_file(socket, &output).map_err(|e| e.to_string())?;
            println!("Saved to {}", output.display());
        }
        Commands::List => unreachable!(),
    }
    Ok(())
}
