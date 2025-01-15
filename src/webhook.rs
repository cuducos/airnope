use actix_web::{
    middleware::Logger,
    web::{self, Bytes},
    App, HttpRequest, HttpResponse, HttpServer,
};
use airnope::{embeddings::Embeddings, is_spam, telegram::Client};
use anyhow::{anyhow, Result};
use futures::try_join;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};
use tokio::sync::Mutex;

const DEFAULT_PORT: u16 = 8000;
const DEFAULT_HOST_IP: &str = "0.0.0.0";
const SECRET_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";

fn random_webhook_secret() -> String {
    let length = thread_rng().gen_range(128..=256);
    (0..length)
        .map(|_| {
            let idx = thread_rng().gen_range(0..SECRET_CHARS.len());
            SECRET_CHARS.chars().nth(idx).unwrap()
        })
        .collect()
}

#[derive(Deserialize, Serialize)]
struct UserOrChat {
    id: i64,
}

#[derive(Deserialize, Serialize)]
struct Message {
    message_id: i64,
    chat: UserOrChat,
    from: UserOrChat,
    text: Option<String>,
}

impl Message {
    async fn is_spam(&self, embeddings: Arc<Mutex<Embeddings>>) -> Result<bool> {
        if let Some(txt) = &self.text {
            match is_spam(&embeddings, txt.as_str()).await {
                Ok(guess) => {
                    return Ok(guess.is_spam);
                }
                Err(e) => {
                    log::error!("Error processing message: {}", e);
                }
            }
        }
        Ok(false)
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
}

impl Update {
    pub fn message(&self) -> Result<&Message> {
        [
            &self.message,
            &self.edited_message,
            &self.channel_post,
            &self.edited_channel_post,
            &self.business_message,
            &self.edited_business_message,
        ]
        .iter()
        .find_map(|&msg| msg.as_ref())
        .ok_or(anyhow!("Could not find message in update payload"))
    }

    async fn is_spam(&self, embeddings: Arc<Mutex<Embeddings>>) -> Result<bool> {
        self.message()?.is_spam(embeddings).await
    }

    async fn mark_as_spam(&self) -> Result<()> {
        self.message()?.mark_as_spam().await
    }
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

async fn handler(
    embeddings: web::Data<Arc<Mutex<Embeddings>>>,
    secret: web::Data<Arc<String>>,
    request: HttpRequest,
    body: Bytes,
) -> HttpResponse {
    let token = request
        .headers()
        .get("X-Telegram-Bot-Api-Secret-Token")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or("");
    if secret.as_str() != token {
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
        Ok(update) => match update.is_spam(embeddings.get_ref().clone()).await {
            Ok(false) => HttpResponse::Ok().finish(),
            Ok(true) => {
                if let Err(e) = update.mark_as_spam().await {
                    log::error!("Error marking message as spam: {}", e);
                    return HttpResponse::InternalServerError().finish();
                }
                HttpResponse::Ok().finish()
            }
            Err(e) => {
                log::error!("Error checking if message is spam: {}", e);
                HttpResponse::InternalServerError().finish()
            }
        },
    }
}

pub async fn run() -> Result<()> {
    let port = env::var("PORT")
        .unwrap_or_else(|_| DEFAULT_PORT.to_string())
        .parse::<u16>()?;
    let secret =
        env::var("TELEGRAM_WEBHOOK_SECRET_TOKEN").unwrap_or_else(|_| random_webhook_secret());
    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?));
    let client = Client::new()?;
    client.delete_webhook().await?;
    client.set_webhook(secret.as_str()).await?;
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(embeddings.clone()))
            .app_data(web::Data::new(Arc::new(secret.clone())))
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
