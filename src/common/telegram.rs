use anyhow::{anyhow, Context, Result};
use reqwest::{Client as ReqwestClient, Url};
use serde::{Deserialize, Serialize};
use std::env;

const TELEGRAM_API_URL: &str = "https://api.telegram.org/bot";
const DEFAULT_MAX_CONNECTIONS: u8 = 100;
const DEFAULT_REACTION: &str = "👀";
const DEFAULT_ALLOWED_UPDATES: &[&str] = &[
    "message",
    "edited_message",
    "channel_post",
    "edited_channel_post",
    "business_message",
    "edited_business_message",
];

#[derive(Serialize)]
struct GetChatMemberPayload {
    chat_id: i64,
    user_id: i64,
}

#[derive(Serialize)]
struct ReactionType {
    #[serde(rename = "type")]
    type_: String,
    emoji: String,
}

#[derive(Serialize)]
struct SetMessageReactionPayload {
    chat_id: i64,
    message_id: i64,
    reaction: Vec<ReactionType>,
    is_big: bool,
}

#[derive(Serialize)]
struct BanChatMemberPayload {
    chat_id: i64,
    user_id: i64,
}

#[derive(Serialize)]
struct DeleteMessagePayload {
    chat_id: i64,
    message_id: i64,
}

#[derive(Serialize)]
pub struct SetWebhookPayload {
    url: String,
    max_connections: u8,
    allowed_updates: Vec<String>,
    secret_token: String,
}

#[derive(Serialize)]
pub struct DeleteWebhookPayload {
    drop_pending_updates: bool,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Payload {
    GetChatMember(GetChatMemberPayload),
    SetMessageReaction(SetMessageReactionPayload),
    BanChatMember(BanChatMemberPayload),
    DeleteMessage(DeleteMessagePayload),
    SetWebhook(SetWebhookPayload),
    DeleteWebhook(DeleteWebhookPayload),
}

#[derive(Deserialize)]
struct ChatMemberResponse {
    status: String,
}

#[derive(Deserialize)]
struct GetChatMemberResponse {
    ok: bool,
    result: ChatMemberResponse,
}

#[derive(Deserialize)]
struct SuccessResponse {
    ok: bool,
    result: bool,
}

enum Response {
    ChatMember(GetChatMemberResponse),
    Success(SuccessResponse),
}

pub struct Client {
    token: String,
    http: ReqwestClient,
}

impl Client {
    pub fn new() -> Result<Self> {
        let token = env::var("TELEGRAM_BOT_TOKEN")
            .map_err(|_| anyhow!("Environment variable TELEGRAM_BOT_TOKEN not found."))?;
        let http = ReqwestClient::new();
        Ok(Client { token, http })
    }

    fn endpoint(&self, payload: &Payload) -> &str {
        match payload {
            Payload::GetChatMember(_) => "getChatMember",
            Payload::SetMessageReaction(_) => "setMessageReaction",
            Payload::BanChatMember(_) => "banChatMember",
            Payload::DeleteMessage(_) => "deleteMessage",
            Payload::SetWebhook(_) => "setWebhook",
            Payload::DeleteWebhook(_) => "deleteWebhook",
        }
    }

    fn url(&self, endpoint: &str) -> Result<Url> {
        let url = format!("{TELEGRAM_API_URL}{}/{}", self.token, endpoint);
        Url::parse(&url).context(format!("Failed to build URL for {endpoint}"))
    }

    async fn post(&self, payload: &Payload) -> Result<Response> {
        let endpoint = self.endpoint(payload);
        let url = self
            .url(endpoint)
            .context(format!("Error creating URL for {endpoint}"))?;
        let response = self
            .http
            .post(url)
            .json(&payload)
            .send()
            .await
            .context(format!("Error in request to {endpoint}"))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .context("Error reading response from {endpoint}")?;
        if !status.is_success() {
            if endpoint == "deleteMessage"
                && (body.contains("message to delete not found")
                    || body.contains("message to react not found")
            || body.contains("can't ban members in private chats"))
            {
                return Ok(Response::Success(SuccessResponse {
                    ok: true,
                    result: true,
                }));
            }
            return Err(anyhow!("Request to {endpoint} failed: [{status}] {body}",));
        }
        match payload {
            Payload::GetChatMember(_) => {
                let chat_member: GetChatMemberResponse = serde_json::from_str(&body).context(
                    format!("Failed to deserialize response from {endpoint}: {body}"),
                )?;
                Ok(Response::ChatMember(chat_member))
            }
            _ => {
                let success: SuccessResponse = serde_json::from_str(&body).context(format!(
                    "Failed to deserialize response from {endpoint}: {body}"
                ))?;
                Ok(Response::Success(success))
            }
        }
    }

    pub async fn is_admin(&self, chat_id: i64, user_id: i64) -> Result<bool> {
        let payload = Payload::GetChatMember(GetChatMemberPayload { chat_id, user_id });
        match self.post(&payload).await? {
            Response::ChatMember(response) => Ok(response.ok
                && (response.result.status == "administrator"
                    || response.result.status == "creator")),
            _ => Err(anyhow!("Unexpected result response for getChatMember")),
        }
    }

    pub async fn set_message_reaction(&self, chat_id: i64, message_id: i64) -> Result<bool> {
        let payload = Payload::SetMessageReaction(SetMessageReactionPayload {
            chat_id,
            message_id,
            reaction: vec![ReactionType {
                type_: "emoji".to_string(),
                emoji: DEFAULT_REACTION.to_string(),
            }],
            is_big: false,
        });
        match self.post(&payload).await? {
            Response::Success(response) => Ok(response.ok && response.result),
            _ => Err(anyhow!("Unexpected result response for setMessageReaction")),
        }
    }

    pub async fn ban_chat_member(&self, chat_id: i64, user_id: i64) -> Result<bool> {
        let payload = Payload::BanChatMember(BanChatMemberPayload { chat_id, user_id });
        match self.post(&payload).await? {
            Response::Success(response) => Ok(response.ok && response.result),
            _ => Err(anyhow!("Unexpected result response for banChatMember")),
        }
    }

    pub async fn delete_message(&self, chat_id: i64, message_id: i64) -> Result<bool> {
        let payload = Payload::DeleteMessage(DeleteMessagePayload {
            chat_id,
            message_id,
        });
        match self.post(&payload).await? {
            Response::Success(response) => Ok(response.ok && response.result),
            _ => Err(anyhow!("Unexpected result response for deleteMessage")),
        }
    }

    pub async fn set_webhook(&self, secret_token: &str) -> Result<bool> {
        let url = env::var("TELEGRAM_WEBHOOK_URL")
            .map_err(|_| anyhow!("Environment variable TELEGRAM_WEBHOOK_URL not found."))?;
        let secret_token = secret_token.to_string();
        log::info!("Setting webhook to: {url}");
        let payload = Payload::SetWebhook(SetWebhookPayload {
            url: url.clone(),
            max_connections: DEFAULT_MAX_CONNECTIONS,
            allowed_updates: DEFAULT_ALLOWED_UPDATES
                .iter()
                .map(|&update| update.to_string())
                .collect(),
            secret_token,
        });
        match self.post(&payload).await? {
            Response::Success(response) => Ok(response.ok && response.result),
            _ => Err(anyhow!("Unexpected result response for setWebhook")),
        }
    }

    pub async fn delete_webhook(&self) -> Result<bool> {
        let payload = Payload::DeleteWebhook(DeleteWebhookPayload {
            drop_pending_updates: false,
        });
        log::info!("Deleting webhook");
        match self.post(&payload).await? {
            Response::Success(response) => Ok(response.ok && response.result),
            _ => Err(anyhow!("Unexpected result response for deleteWebhook")),
        }
    }
}
