use anyhow::Result;
use re::RegularExpression;
use std::env::args;
use teloxide::prelude::{Bot, Message};
use teloxide::respond;
use teloxide::types::MessageKind;
use zsc::ZeroShotClassification;

mod embeddings;
mod re;
mod repl;
mod telegram;
mod zsc;

#[derive(Clone)]
struct Pipeline {
    regex: RegularExpression,
    zero_shot: ZeroShotClassification,
}

impl Pipeline {
    async fn new() -> Result<Self> {
        Ok(Self {
            regex: RegularExpression::new().await?,
            zero_shot: ZeroShotClassification::new().await?,
        })
    }

    async fn is_spam(&self, txt: String) -> Result<bool> {
        Ok(self.regex.is_spam(&txt).await? && self.zero_shot.is_spam(&txt).await?)
    }
}

async fn process_message(bot: &Bot, msg: &Message) {
    if let MessageKind::Common(_) = &msg.kind {
        if let Some(txt) = msg.text() {
            match Pipeline::new().await {
                Ok(pipeline) => {
                    let result = pipeline.is_spam(txt.to_string()).await;
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
                Err(e) => log::error!("Error creating the pipeline: {:?}", e),
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init(); // based on RUST_LOG environment variable

    std::thread::spawn(|| {
        if let Err(e) = embeddings::serve() {
            log::error!("Error spawning the embedding server: {}", e);
        }
    });
    embeddings::wait_until_ready().await?;

    if args().any(|arg| arg == "--download") {
        Pipeline::new().await?;
        return Ok(());
    }

    if args().any(|arg| arg == "--repl") {
        repl::run().await?;
        return Ok(());
    }

    let bot = Bot::from_env(); // requires TELOXIDE_TOKEN environment variable
    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        process_message(&bot, &msg).await;
        respond(())
    })
    .await;

    Ok(())
}
