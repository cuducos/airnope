use crate::embeddings::Embeddings;
use anyhow::Result;
use clap::ValueEnum;
use std::{env, sync::Arc, time::Duration};
use teloxide::{
    dispatching::{DefaultKey, UpdateFilterExt},
    dptree,
    prelude::{Bot, Dispatcher, LoggingErrorHandler, Message, Request, Requester},
    respond,
    types::{AllowedUpdate, ChatMemberStatus, MessageKind, ReactionType, Update},
    update_listeners::webhooks,
    RequestError,
};
use tokio::{sync::Mutex, time::sleep};
use url::Url;

const DEFAULT_PORT: u16 = 8000;
const DEFAULT_HOST_IP: [u8; 4] = [0, 0, 0, 0];

#[derive(Clone, Debug, ValueEnum)]
pub enum AirNope {
    LongPooling,
    Webhook,
}

pub async fn delete_message(bot: &Bot, msg: &Message) -> Result<()> {
    bot.delete_message(msg.chat.id, msg.id).send().await?;
    Ok(())
}

pub async fn ban_user(bot: &Bot, msg: &Message) -> Result<()> {
    if let Some(user) = &msg.from {
        bot.kick_chat_member(msg.chat.id, user.id).send().await?;
    }
    Ok(())
}

pub async fn is_admin(bot: &Bot, msg: &Message) -> bool {
    if let Some(user) = &msg.from {
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

async fn react(bot: &Bot, msg: &Message) {
    let eyes = ReactionType::Emoji {
        emoji: "👀".to_string(),
    };
    let mut request = bot.set_message_reaction(msg.chat.id, msg.id);
    request.reaction = Some(vec![eyes]);
    if let Err(e) = request.send().await {
        log::error!("Error reacting to spam message: {:?}", e);
    };
}

async fn process_message(
    bot: &Bot,
    embeddings: &Arc<Mutex<Embeddings>>,
    msg: &Message,
) -> Result<(), RequestError> {
    if let MessageKind::Common(_) = &msg.kind {
        if let Some(txt) = msg.text() {
            let result = crate::is_spam(embeddings, txt).await;
            if let Err(e) = result {
                log::error!("Error in the pipeline: {:?}", e);
                return respond(());
            }
            if let Ok(false) = result {
                return respond(());
            }
            if is_admin(bot, msg).await {
                react(bot, msg).await;
                return respond(());
            }
            if let Err(e) = tokio::try_join!(delete_message(bot, msg), ban_user(bot, msg),) {
                log::error!("Error handling spam: {:?}", e);
            }
        }
    }
    respond(())
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
            return Err(anyhow::anyhow!("No HOST environment variable set."));
        }
    };
    let url = Url::parse(format!("https://{host}/webhook").as_str())?;
    let opts =
        webhooks::Options::new((DEFAULT_HOST_IP, port).into(), url.clone()).max_connections(32);
    let mut webhook = bot.set_webhook(url);
    webhook.allowed_updates = Some(vec![AllowedUpdate::Message, AllowedUpdate::EditedMessage]);
    webhook.send().await?;
    sleep(Duration::from_secs(2)).await; // Teloxide also sends setWebhook, this avoids a 429-like error
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
    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(process_message))
        .branch(Update::filter_edited_message().endpoint(process_message));
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
