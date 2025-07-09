use actix_web::{
    middleware::Logger,
    web::{self, Bytes},
    App, HttpRequest, HttpResponse, HttpServer,
};
use airnope::{embeddings::Embeddings, is_spam, telegram::Client};
use anyhow::{anyhow, Result};
use futures::try_join;
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};
use tokio::sync::Mutex;

const DEFAULT_PORT: u16 = 8000;
const DEFAULT_HOST_IP: &str = "0.0.0.0";
const SECRET_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
const DEFAULT_AIRNOPE_HANDLE: &str = "@AirNope_bot";

fn random_webhook_secret() -> String {
    let length = rng().random_range(128..=256);
    (0..length)
        .map(|_| {
            let idx = rng().random_range(0..SECRET_CHARS.len());
            SECRET_CHARS.chars().nth(idx).unwrap()
        })
        .collect()
}

#[derive(Clone)]
struct Settings {
    handle: String,
    secret: String,
}

impl Settings {
    fn new() -> Settings {
        let secret = env::var("TELEGRAM_WEBHOOK_SECRET_TOKEN").unwrap_or(random_webhook_secret());
        let handle = env::var("AIRNOPE_HANDLE").unwrap_or(DEFAULT_AIRNOPE_HANDLE.to_string());
        Settings { handle, secret }
    }
}

#[derive(Deserialize, Serialize)]
struct UserOrChat {
    id: i64,
}

#[derive(Deserialize, Serialize)]
struct Chat {
    title: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct SenderUser {
    username: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct ForwardOrigin {
    chat: Option<Chat>,
    sender_user: Option<SenderUser>,
}

#[derive(Deserialize, Serialize)]
struct InlineKeyboard {
    text: Option<String>,
    url: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct ReplyMarkup {
    inline_keyboard: Option<Vec<Vec<InlineKeyboard>>>,
}

#[derive(Deserialize, Serialize)]
struct Message {
    message_id: i64,
    chat: UserOrChat,
    from: UserOrChat,
    text: Option<String>,
    caption: Option<String>,
    reply_to_message: Option<Box<Message>>,
    forward_origin: Option<ForwardOrigin>,
    forward_from_chat: Option<Chat>,
    reply_markup: Option<ReplyMarkup>,
}

impl Message {
    fn contents(&self) -> Option<String> {
        let text = self
            .text
            .as_deref()
            .into_iter()
            .chain(self.caption.as_deref());
        let forward = self
            .forward_origin
            .as_ref()
            .and_then(|f| f.chat.as_ref().and_then(|c| c.title.as_deref()))
            .into_iter()
            .chain(
                self.forward_from_chat
                    .as_ref()
                    .and_then(|f| f.title.as_deref()),
            );
        let buttons = self
            .reply_markup
            .as_ref()
            .into_iter()
            .flat_map(|reply_markup| reply_markup.inline_keyboard.as_ref())
            .flat_map(|keyboards| keyboards.iter())
            .flatten()
            .flat_map(|keyboard| {
                keyboard
                    .text
                    .as_deref()
                    .into_iter()
                    .chain(keyboard.url.as_deref())
            });
        let merged = text.chain(forward).chain(buttons).collect::<Vec<&str>>();
        if merged.is_empty() {
            return None;
        }

        Some(merged.join("\n\n"))
    }

    async fn is_spam(&self, embeddings: Arc<Mutex<Embeddings>>) -> Result<bool> {
        if let Some(sender) = self
            .forward_origin
            .as_ref()
            .and_then(|fw| fw.sender_user.as_ref())
            .and_then(|u| u.username.as_ref())
        {
            if sender == "safeguard" {
                return Ok(true);
            }
        }
        if let Some(txt) = &self.contents() {
            match is_spam(&embeddings, txt.as_str()).await {
                Ok(guess) => {
                    return Ok(guess.is_spam);
                }
                Err(e) => {
                    log::error!("Error processing message: {e}");
                }
            }
        }
        Ok(false)
    }

    async fn acknowledge(&self) -> Result<()> {
        let client = Client::new()?;
        client
            .set_message_reaction(self.chat.id, self.message_id)
            .await?;
        Ok(())
    }

    async fn mark_as_spam(&self) -> Result<()> {
        let client = Client::new()?;
        if client.is_admin(self.chat.id, self.from.id).await? {
            client
                .set_message_reaction(self.chat.id, self.message_id)
                .await?;
            return Ok(());
        }
        try_join!(
            client.delete_message(self.chat.id, self.message_id),
            client.ban_chat_member(self.chat.id, self.from.id)
        )?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
struct Update {
    message: Option<Message>,
    edited_message: Option<Message>,
    channel_post: Option<Message>,
    edited_channel_post: Option<Message>,
    business_message: Option<Message>,
    edited_business_message: Option<Message>,
    current_bot_handle: Option<String>,
}

impl Update {
    pub async fn message(&self) -> Result<&Message> {
        if self.is_tagging_airnope().await {
            if let Some(msg) = self.message.as_ref() {
                if let Some(replying_to) = msg.reply_to_message.as_ref() {
                    return Ok(replying_to);
                }
            }
        }
        [
            &self.edited_message,
            &self.message,
            &self.edited_channel_post,
            &self.channel_post,
            &self.edited_business_message,
            &self.business_message,
        ]
        .iter()
        .find_map(|&msg| msg.as_ref())
        .ok_or(anyhow!("Could not find message in update payload"))
    }

    async fn is_spam(&self, embeddings: Arc<Mutex<Embeddings>>) -> Result<bool> {
        self.message().await?.is_spam(embeddings).await
    }

    async fn mark_as_spam(&self) -> Result<()> {
        self.message().await?.mark_as_spam().await
    }

    async fn is_tagging_airnope(&self) -> bool {
        let mut result = false;
        if let Some(msg) = self.message.as_ref() {
            if let Some(handle) = self.current_bot_handle.as_ref() {
                result = msg
                    .text
                    .as_ref()
                    .is_some_and(|txt| txt.to_lowercase().trim() == handle.to_lowercase());
                if result {
                    if let Err(error) = msg.acknowledge().await {
                        log::error!("Error reacting to message tagging AirNope: {error}")
                    }
                }
            }
        }
        result
    }
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

async fn handler(
    embeddings: web::Data<Arc<Mutex<Embeddings>>>,
    settings: web::Data<Arc<Settings>>,
    request: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    let token = request
        .headers()
        .get("X-Telegram-Bot-Api-Secret-Token")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");
    if settings.secret.as_str() != token {
        return HttpResponse::Unauthorized().finish();
    }
    match serde_json::from_slice::<Update>(&body) {
        Err(e) => {
            log::error!(
                "Error parsing update: {}\n{}",
                e,
                String::from_utf8_lossy(&body)
            );
            HttpResponse::BadRequest().finish()
        }
        Ok(mut update) => {
            update.current_bot_handle = Some(settings.handle.clone());
            match update.is_spam(embeddings.get_ref().clone()).await {
                Ok(false) => HttpResponse::Ok().finish(),
                Ok(true) => {
                    if let Err(e) = update.mark_as_spam().await {
                        log::error!("Error marking message as spam: {e}");
                        return HttpResponse::InternalServerError().finish();
                    }
                    HttpResponse::Ok().finish()
                }
                Err(e) => {
                    log::error!("Error checking if message is spam: {e}");
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
    }
}

pub async fn run() -> Result<()> {
    let port = env::var("PORT")
        .unwrap_or(DEFAULT_PORT.to_string())
        .parse::<u16>()?;
    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?));
    let client = Client::new()?;
    let settings = Settings::new();
    client.delete_webhook().await?;
    client.set_webhook(settings.secret.as_str()).await?;
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(embeddings.clone()))
            .app_data(web::Data::new(Arc::new(settings.clone())))
            .route("/", web::post().to(handler))
            .route("/health", web::get().to(health))
    })
    .workers(32)
    .bind((DEFAULT_HOST_IP, port))?
    .run()
    .await?;
    Ok(())
}

pub async fn remove() -> Result<()> {
    let client = Client::new()?;
    client.delete_webhook().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_deserialize_message() {
        let data = fs::read_to_string("test_data/message_administrator.json").unwrap();
        let message: Message = serde_json::from_str(&data).unwrap();
        assert_eq!(message.message_id, 88220);
        assert!(message
            .forward_origin
            .and_then(|fw| fw.sender_user)
            .and_then(|u| u.username)
            .is_none());
    }

    #[test]
    fn test_deserialize_message_with_sender() {
        let data = fs::read_to_string("test_data/message_safeguard.json").unwrap();
        let message: Message = serde_json::from_str(&data).unwrap();
        assert_eq!(message.message_id, 204091);
        assert_eq!(
            message
                .forward_origin
                .and_then(|fw| fw.sender_user)
                .and_then(|u| u.username)
                .unwrap(),
            "safeguard"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_message_from_safeguard_is_spam() {
        let embeddings = Arc::new(Mutex::new(Embeddings::new().await.unwrap()));
        let data = fs::read_to_string("test_data/message_safeguard.json").unwrap();
        let message: Message = serde_json::from_str(&data).unwrap();
        assert!(message.is_spam(embeddings).await.unwrap());
    }
}
