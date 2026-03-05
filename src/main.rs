use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, error};

mod cli;
mod config;
mod daemon;
mod log;
mod process;

use cli::commands;

#[derive(Parser)]
#[command(name = "pm2")]
#[command(about = "Production Process Manager for Node.js and other applications")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a process
    Start {
        /// Script or configuration file to start
        script: String,
        /// Process name
        #[arg(short, long)]
        name: Option<String>,
        /// Number of instances
        #[arg(short, long)]
        instances: Option<usize>,
        /// Run in cluster mode
        #[arg(long)]
        cluster: bool,
        /// Watch for file changes
        #[arg(long)]
        watch: bool,
        /// Max memory restart
        #[arg(long)]
        max_memory_restart: Option<String>,
        /// Log file path
        #[arg(long)]
        log: Option<String>,
        /// Error log file path
        #[arg(long)]
        error_log: Option<String>,
        /// Environment variables (format: KEY=value)
        #[arg(short, long)]
        env: Vec<String>,
    },
    /// Stop a process
    Stop {
        /// Process name or id
        name: String,
    },
    /// Restart a process
    Restart {
        /// Process name or id
        name: String,
    },
    /// Delete a process
    Delete {
        /// Process name or id
        name: String,
    },
    /// List all processes
    List,
    /// Show process details
    Show {
        /// Process name or id
        name: String,
    },
    /// Monitor processes
    Monit,
    /// View logs
    Logs {
        /// Process name (optional)
        name: Option<String>,
        /// Number of lines to show
        #[arg(short, long, default_value = "20")]
        lines: usize,
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
    /// Flush logs
    Flush,
    /// Reload configuration
    Reload {
        /// Process name or id
        name: String,
    },
    /// Save process list
    Save,
    /// Resurrect saved processes
    Resurrect,
    /// Startup script generation
    Startup {
        /// Platform (ubuntu, centos, systemd, etc.)
        #[arg(short, long)]
        platform: Option<String>,
    },
    /// Stop daemon
    Kill,
    /// Update PM2 daemon
    Update,
    /// Rotate logs
    Rotate {
        /// Process name (optional, rotates all if not specified)
        name: Option<String>,
    },
    /// List log files
    LogFiles {
        /// Process name (optional, lists all if not specified)
        name: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            script,
            name,
            instances,
            cluster,
            watch,
            max_memory_restart,
            log,
            error_log,
            env,
        } => {
            info!("Starting process: {}", script);
            commands::start(
                script,
                name,
                instances,
                cluster,
                watch,
                max_memory_restart,
                log,
                error_log,
                env,
            )
            .await?;
        }
        Commands::Stop { name } => {
            info!("Stopping process: {}", name);
            commands::stop(&name).await?;
        }
        Commands::Restart { name } => {
            info!("Restarting process: {}", name);
            commands::restart(&name).await?;
        }
        Commands::Delete { name } => {
            info!("Deleting process: {}", name);
            commands::delete(&name).await?;
        }
        Commands::List => {
            commands::list().await?;
        }
        Commands::Show { name } => {
            commands::show(&name).await?;
        }
        Commands::Monit => {
            commands::monit().await?;
        }
        Commands::Logs { name, lines, follow } => {
            commands::logs(name.as_deref(), lines, follow).await?;
        }
        Commands::Flush => {
            commands::flush().await?;
        }
        Commands::Reload { name } => {
            commands::reload(&name).await?;
        }
        Commands::Save => {
            commands::save().await?;
        }
        Commands::Resurrect => {
            commands::resurrect().await?;
        }
        Commands::Startup { platform } => {
            commands::startup(platform).await?;
        }
        Commands::Kill => {
            commands::kill().await?;
        }
        Commands::Update => {
            commands::update().await?;
        }
        Commands::Rotate { name } => {
            info!("Rotating logs for: {:?}", name);
            commands::rotate_logs(name.as_deref()).await?;
        }
        Commands::LogFiles { name } => {
            commands::log_files(name.as_deref()).await?;
        }
    }

    Ok(())
}
