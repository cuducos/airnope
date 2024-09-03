use airnope::telegram::AirNope;
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
    Bot {
        #[clap(long, short)]
        mode: Option<AirNope>,
    },
    /// Runs benchmark of the zero-shot classification model (accepts labels as arguments)
    Bench {
        /// One or more label sets to benchmark  (separate different labels in a set using commas)
        #[clap(required = true)]
        label: Vec<String>,

        /// Skip summary in case the score is too close to the threshold
        #[clap(long, short)]
        skip_summary: bool,

        /// Threshold difference to ask for summary instead
        #[clap(long, short, default_value_t = 0.05)]
        threshold_difference: f32,
    },
    /// Start the REPL for individual message testing
    Repl,
    /// Start the web API used for the online playground
    Demo,
    /// Cache the embedding model
    Download,
    /// Clean `rust-bert` cache
    CleanCache {
        /// Show the amount of space that would be freed, without deleting any file or directory
        #[clap(long, short, default_value_t = false)]
        dry_run: bool,
    },
}
