use anyhow::{anyhow, Result};
use embeddings::Embeddings;
use re::RegularExpression;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use zsc::ZeroShotClassification;

mod bench;
mod embeddings;
mod re;
mod repl;
mod telegram;
mod web;
mod zsc;

const HELP: &str = "Usage: airnope [ --repl | --web | --download | --bench ]";

pub async fn is_spam(embeddings: &Arc<Mutex<Embeddings>>, txt: &str) -> Result<bool> {
    let regex = RegularExpression::new().await?;
    if !regex.is_spam(txt).await? {
        return Ok(false);
    }
    let zero_shot = ZeroShotClassification::new(embeddings).await?;
    zero_shot.is_spam(embeddings, txt).await
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init(); // based on RUST_LOG environment variable
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 && args[1] != "--bench" {
        return Err(anyhow!("{}", HELP));
    }
    if args.len() >= 2 {
        match args[1].as_str() {
            "--download" => return embeddings::download().await,
            "--repl" => return repl::run().await,
            "--web" => return web::run().await,
            "--bench" => return bench::run().await,
            unknown => return Err(anyhow!("Unknown option: {}\n{}", unknown, HELP)),
        }
    }
    telegram::run().await
}
