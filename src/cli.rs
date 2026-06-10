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
    /// Clean model cache
    CleanCache {
        /// Show the amount of space that would be freed, without deleting any file or directory
        #[clap(long, short, default_value_t = false)]
        dry_run: bool,
    },
    /// Remove the bot webhook from Telegram's server
    RemoveWebhook,
    /// Evaluate classifier accuracy against a Kaggle dataset
    #[command(group(
        clap::ArgGroup::new("expected")
            .required(true)
            .args(["spam", "not_spam"])
    ))]
    Kaggle {
        /// Kaggle dataset in 'owner/name' format
        dataset: String,

        /// CSV column containing the text to classify
        #[arg(long)]
        column: String,

        /// Optional CSV column to filter by
        #[arg(long)]
        filter_column: Option<String>,

        /// Only include rows where filter-column matches this value
        #[arg(long)]
        filter_value: Option<String>,

        /// Expect all rows to be spam
        #[arg(long, conflicts_with = "not_spam")]
        spam: bool,

        /// Expect all rows to NOT be spam
        #[arg(long, conflicts_with = "spam")]
        not_spam: bool,
    },
}
