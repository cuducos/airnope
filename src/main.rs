use airnope::embeddings;
use airnope::telegram;
use airnope::telegram::AirNope;
use anyhow::{anyhow, Result};
use std::env;

const HELP: &str = "Usage: airnope [ --only-download-model | --web ]";

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init(); // based on RUST_LOG environment variable
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        return Err(anyhow!(HELP));
    }
    let mut mode = AirNope::LongPooling;
    if args.len() == 2 {
        match args[1].as_str() {
            "--only-download-model" => return embeddings::download().await,
            "--web" => {
                mode = AirNope::Webhook;
            }
            unknown => return Err(anyhow!("Unknown option: {}\n{}", unknown, HELP)),
        }
    }
    telegram::run(mode).await
}
