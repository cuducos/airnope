use crate::embeddings::Embeddings;
use anyhow::Result;
use std::sync::Arc;
use teloxide::dispatching::UpdateFilterExt;
use teloxide::prelude::{Bot, Dispatcher, Message, Request};
use teloxide::requests::Requester;
use teloxide::types::ChatMemberStatus;
use teloxide::types::{MessageKind, Update};
use teloxide::{dptree, respond};
use tokio::sync::Mutex;

pub async fn delete_message(bot: &Bot, msg: &Message) -> Result<()> {
    bot.delete_message(msg.chat.id, msg.id).send().await?;
    Ok(())
}

pub async fn ban_user(bot: &Bot, msg: &Message) -> Result<()> {
    if let Some(user) = msg.from() {
        bot.kick_chat_member(msg.chat.id, user.id).send().await?;
    }
    Ok(())
}

pub async fn is_admin(bot: &Bot, msg: &Message) -> bool {
    if let Some(user) = &msg.from() {
        if let Ok(member) = bot.get_chat_member(msg.chat.id, user.id).await {
            match member.status() {
                ChatMemberStatus::Administrator => return true,
                ChatMemberStatus::Owner => return true,
                _ => return false,
            }
        }
    }
    false
}

async fn process_message(bot: &Bot, embeddings: &Arc<Mutex<Embeddings>>, msg: &Message) {
    if let MessageKind::Common(_) = &msg.kind {
        if let Some(txt) = msg.text() {
            let result = crate::is_spam(embeddings, txt).await;
            if let Err(e) = result {
                log::error!("Error in the pipeline: {:?}", e);
                return;
            }
            if let Ok(false) = result {
                return;
            }
            if is_admin(bot, msg).await {
                return;
            }
            if let Err(e) = tokio::try_join!(delete_message(bot, msg), ban_user(bot, msg),) {
                log::error!("Error handling spam: {:?}", e);
            }
        }
    }
}

pub async fn run() -> Result<()> {
    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?));
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
