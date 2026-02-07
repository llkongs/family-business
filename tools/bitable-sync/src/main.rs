mod config;
mod feishu;
mod git;
mod models;
mod output;
mod setup;
mod sync;
mod transform;
mod video;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bitable-sync")]
#[command(about = "Sync product data from Feishu Bitable to family-business website")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync data from bitable to local files (and optionally push)
    Sync {
        /// Only read and transform data, don't write files
        #[arg(long)]
        dry_run: bool,

        /// Write files but don't git commit/push
        #[arg(long)]
        no_push: bool,
    },

    /// List all tables in the bitable app (for configuration)
    ListTables,

    /// Verify configuration and connectivity
    Check,

    /// Create all tables in bitable from scratch (destructive!)
    Setup,

    /// Add the slogans table to an existing bitable app (non-destructive)
    AddSlogansTable,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let config = config::Config::load()?;

    match cli.command {
        Commands::Sync { dry_run, no_push } => {
            config.validate()?;
            let opts = sync::SyncOptions { dry_run, no_push };
            sync::run_sync(&config, &opts).await?;
        }
        Commands::ListTables => {
            sync::list_tables(&config).await?;
        }
        Commands::Check => {
            sync::check_config(&config).await?;
        }
        Commands::Setup => {
            setup::setup_tables(&config).await?;
        }
        Commands::AddSlogansTable => {
            setup::create_slogans_table(&config).await?;
        }
    }

    Ok(())
}
