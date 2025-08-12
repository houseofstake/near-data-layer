mod config;
mod database;
mod indexer;
mod metrics;
mod processor;

use clap::{Parser, Subcommand};
use config::Settings;
use indexer::Indexer;

#[derive(Parser)]
#[command(name = "near-indexer")]
#[command(about = "NEAR blockchain indexer for HOS contracts")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the indexer
    Start {
        /// Start block height (default: from config)
        #[arg(long)]
        start_block: Option<u64>,
        
        /// Number of threads (default: from config)
        #[arg(long)]
        num_threads: Option<u64>,
    },
    
    /// Initialize database tables
    Init,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration first to get log level
    let mut settings = Settings::new().expect("Failed to load configuration. Please ensure config/testnet.toml or mainnet.toml exists for the configured chain ID.");

    tracing_subscriber::fmt()
        .with_env_filter("neardata-fetcher=debug")
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { start_block, num_threads } => {
            // Override config with CLI arguments
            if let Some(threads) = num_threads {
                settings.num_threads = threads;
            }

            tracing::info!("Starting NEAR blockchain indexer");

            let mut indexer = Indexer::new(settings).await?;
            indexer.initialize().await?;
            indexer.start(start_block).await?;
        }
        
        Commands::Init => {
            tracing::info!("Initializing database tables");
            
            let indexer = Indexer::new(settings).await?;
            indexer.initialize().await?;
            tracing::info!("Database initialization completed");
        }
    }

    Ok(())
} 