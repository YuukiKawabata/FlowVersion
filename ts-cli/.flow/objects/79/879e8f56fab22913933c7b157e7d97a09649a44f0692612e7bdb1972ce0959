use clap::{Parser, Subcommand};
use tracing::{info, error};
use tracing_subscriber;

mod cli;
mod core;
mod error;
mod storage;
mod utils;

use cli::commands::handle_command;
use error::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "flow")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new FlowVersion repository
    Init {
        /// Repository name
        #[arg(long)]
        name: Option<String>,
        
        /// AI mode (local, openai, claude)
        #[arg(long, default_value = "local")]
        ai_mode: String,
    },
    /// Add files to staging area with optional intention
    Add {
        /// Files to add
        files: Vec<String>,
        
        /// Specify intention for the changes
        #[arg(long)]
        intention: Option<String>,
    },
    /// Create an intent-based commit
    Commit {
        /// Main goal/intention for this commit
        #[arg(long)]
        intention: String,
        
        /// Context or background for the change
        #[arg(long)]
        context: Option<String>,
        
        /// Expected impact of the change
        #[arg(long)]
        impact: Option<String>,
        
        /// Confidence score (0.0-1.0)
        #[arg(long)]
        confidence: Option<f32>,
        
        /// Let AI suggest the intention
        #[arg(long)]
        ai_suggest: bool,
    },
    /// Show commit history
    Log {
        /// Show one line per commit
        #[arg(long)]
        oneline: bool,
        
        /// Show commit graph
        #[arg(long)]
        graph: bool,
        
        /// Show intentions in the log
        #[arg(long)]
        intentions: bool,
    },
    /// Show changes in a commit
    Show {
        /// Commit ID to show
        commit_id: String,
    },
    /// Show differences between commits or working directory
    Diff {
        /// First commit ID (optional)
        commit1: Option<String>,
        
        /// Second commit ID (optional)
        commit2: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(log_level))
        .init();
    
    info!("FlowVersion starting");
    
    match handle_command(cli.command).await {
        Ok(_) => {
            info!("Command completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Command failed: {}", e);
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}