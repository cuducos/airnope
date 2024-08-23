use crate::embeddings::Embeddings;
use anyhow::Result;
use std::{env, sync::Arc};
use teloxide::{
    dispatching::{DefaultKey, UpdateFilterExt},
    dptree,
    prelude::{Bot, Dispatcher, LoggingErrorHandler, Message, Request, Requester},
    respond,
    types::{ChatMemberStatus, MessageKind, Update},
    update_listeners::webhooks,
    RequestError,
};
use tokio::sync::Mutex;

const DEFAULT_PORT: u16 = 8000;
const DEFAULT_HOST_IP: [u8; 4] = [0, 0, 0, 0];

pub enum AirNope {
    LongPooling,
    Webhook,
}

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

async fn webhook(
    bot: Bot,
    mut dispatcher: Dispatcher<Bot, RequestError, DefaultKey>,
) -> Result<()> {
    let port: u16 = match env::var("PORT") {
        Ok(p) => p.parse()?,
        Err(_) => {
            log::info!(
                "No PORT environment variable set. Using default port {}.",
                DEFAULT_PORT
            );
            DEFAULT_PORT
        }
    };
    let host = match env::var("HOST") {
        Ok(h) => h,
        Err(_) => {
            return Err(anyhow::anyhow!("No HOST_NAME environment variable set."));
        }
    };
    let opts = webhooks::Options::new(
        (DEFAULT_HOST_IP, port).into(),
        format!("https://{host}/webhook").parse()?,
    )
    .max_connections(16); // up to 100
    dispatcher
        .dispatch_with_listener(
            webhooks::axum(bot, opts).await?,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
    Ok(())
}

pub async fn run(mode: AirNope) -> Result<()> {
    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?));
    let bot = Bot::from_env(); // requires TELOXIDE_TOKEN environment variable
    let handler = Update::filter_message().endpoint(
        |bot: Bot, embeddings: Arc<Mutex<Embeddings>>, msg: Message| async move {
            process_message(&bot, &embeddings, &msg).await;
            respond(())
        },
    );
    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![embeddings])
        .enable_ctrlc_handler()
        .build();
    log::info!("Starting AirNope bot...");
    match mode {
        AirNope::LongPooling => dispatcher.dispatch().await,
        AirNope::Webhook => webhook(bot, dispatcher).await?,
    }
    Ok(())
}
