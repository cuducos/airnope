use anyhow::{anyhow, Result};
use embeddings::Embeddings;
use re::RegularExpression;
use std::env;
use std::sync::Arc;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::prelude::{Bot, Dispatcher, Message};
use teloxide::types::{MessageKind, Update};
use teloxide::{dptree, respond};
use tokio::sync::Mutex;
use zsc::ZeroShotClassification;

mod embeddings;
mod re;
mod repl;
mod telegram;
mod zsc;

const HELP: &str = "Usage: airnope [ --repl | --download ]";

pub async fn is_spam(embeddings: &Arc<Mutex<Embeddings>>, txt: &str) -> Result<bool> {
    let regex = RegularExpression::new().await?;
    if !regex.is_spam(txt).await? {
        return Ok(false);
    }
    let zero_shot = ZeroShotClassification::new(embeddings).await?;
    zero_shot.is_spam(embeddings, txt).await
}

async fn process_message(bot: &Bot, embeddings: &Arc<Mutex<Embeddings>>, msg: &Message) {
    if let MessageKind::Common(_) = &msg.kind {
        if let Some(txt) = msg.text() {
            let result = is_spam(embeddings, txt).await;
            if let Err(e) = result {
                log::error!("Error in the pipeline: {:?}", e);
                return;
            }
            if let Ok(false) = result {
                return;
            }
            if telegram::is_admin(bot, msg).await {
                return;
            }
            if let Err(e) = tokio::try_join!(
                telegram::delete_message(bot, msg),
                telegram::ban_user(bot, msg),
            ) {
                log::error!("Error handling spam: {:?}", e);
            }
        }
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init(); // based on RUST_LOG environment variable
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        return Err(anyhow!("{}", HELP));
    }
    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?));
    if args.len() == 2 {
        match args[1].as_str() {
            "--download" => return Ok(()),
            "--repl" => return repl::run(&embeddings).await,
            unknown => return Err(anyhow!("Unknown option: {}\n{}", unknown, HELP)),
        }
    }
    let bot = Bot::from_env(); // requires TELOXIDE_TOKEN environment variable
    let handler = Update::filter_message().endpoint(
        |bot: Bot, embeddings: Arc<Mutex<Embeddings>>, msg: Message| async move {
            process_message(&bot, &embeddings, &msg).await;
            respond(())
        },
    );
    log::info!("Starting AirNope bot...");
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![embeddings])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}
