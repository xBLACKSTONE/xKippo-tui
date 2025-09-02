mod app;
mod config;
mod core;
mod data;
mod plugins;
mod ui;
mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use log::{info, LevelFilter};

/// Command line arguments for xKippo
#[derive(Parser, Debug)]
#[clap(
    name = "xkippo-tui",
    author = "xKippo Team",
    version,
    about = "A TUI monitoring and management system for Cowrie honeypots"
)]
struct Args {
    /// Path to configuration file
    #[clap(short, long, value_name = "FILE")]
    config: Option<std::path::PathBuf>,

    /// Increase verbosity (can be used multiple times)
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Enable debug mode
    #[clap(short, long)]
    debug: bool,

    /// Run setup script to configure xKippo
    #[clap(long)]
    setup: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Configure logger
    let log_level = match args.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp_secs()
        .init();

    info!("Starting xKippo-tui v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config_path = args.config;
    let config = config::Config::load(config_path.as_deref())
        .context("Failed to load configuration")?;

    // Run setup mode if requested
    if args.setup {
        return run_setup(&config).await;
    }

    // Initialize the application
    let app = app::App::new(config).await?;

    // Start the UI
    ui::start_ui(app).context("UI error")?;

    info!("xKippo-tui shut down successfully");
    Ok(())
}

/// Run the setup wizard
async fn run_setup(config: &config::Config) -> Result<()> {
    println!("Running xKippo setup...");
    
    // TODO: Implement setup script that:
    // 1. Detects cowrie installation
    // 2. Finds log locations
    // 3. Configures monitoring options
    // 4. Sets up GeoIP database
    // 5. Saves configuration

    println!("Setup completed successfully. Run xKippo-tui to start monitoring.");
    Ok(())
}