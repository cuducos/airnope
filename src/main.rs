use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use std::env;

mod bench;
mod cache;
mod cli;
mod repl;
mod webhook;

const DEFAULT_LOG_LEVEL: &str = "INFO";

fn init_log() -> Result<()> {
    let mut default_used = false;
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", DEFAULT_LOG_LEVEL);
        default_used = true;
    }
    env_logger::init();
    if default_used {
        log::info!(
            "No RUST_LOG environment variable found, using default log level: {}",
            DEFAULT_LOG_LEVEL
        );
    }
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    init_log()?;
    let args = Cli::parse();
    match args.command {
        Commands::Bot => webhook::run().await,
        Commands::RemoveWebhook => webhook::remove().await,
        Commands::Repl => repl::run().await,
        Commands::Download => cache::download_all().await,
        Commands::Bench { label, pattern } => bench::run(label, pattern).await,
        Commands::CleanCache { dry_run } => cache::clean_rust_bert_cache(dry_run).await,
    }
}
