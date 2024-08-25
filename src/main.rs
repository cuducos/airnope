use airnope::embeddings;
use airnope::telegram;
use airnope::telegram::AirNope;
use anyhow::{anyhow, Result};
use std::env;

const HELP: &str = "Usage: airnope [ --only-download-model | --web | --pool ]";

fn detect_mode(args: Vec<String>) -> AirNope {
    match args.get(1).map(|s| s.as_str()) {
        Some("--pool") => AirNope::LongPooling,
        Some("--web") => AirNope::Webhook,
        _ => {
            if env::var("HOST").is_ok() && env::var("PORT").is_ok() {
                log::info!("HOST and PORT are set, so using `--web`");
                AirNope::Webhook
            } else {
                AirNope::LongPooling
            }
        }
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init(); // based on RUST_LOG environment variable
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        return Err(anyhow!(HELP));
    }
    if args.len() == 2 {
        match args[1].as_str() {
            "--only-download-model" => return embeddings::download().await,
            "--pool" => (),
            "--web" => (),
            unknown => return Err(anyhow!("Unknown option: {}\n{}", unknown, HELP)),
        }
    }
    let mode = detect_mode(args);
    telegram::run(mode).await
}
