use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "airnope")]
#[command(about = "Keep your Telegram groups free of crypto airdrop spam", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start AirNope bot
    Bot,
    /// Runs benchmark of the zero-shot classification model (accepts labels as arguments)
    Bench {
        /// One or more label sets to benchmark  (separate different labels in a set using commas)
        label: Option<Vec<String>>,

        /// Only runs the benchmark in files that match that pattern
        #[arg(short, long)]
        pattern: Option<String>,
    },
    /// Start the REPL for individual message testing
    Repl,
    /// Cache the embedding model
    Download,
    /// Clean `rust-bert` cache
    CleanCache {
        /// Show the amount of space that would be freed, without deleting any file or directory
        #[clap(long, short, default_value_t = false)]
        dry_run: bool,
    },
    /// Remove the bot webhook from Telegram's server
    RemoveWebhook,
}
