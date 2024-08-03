use anyhow::Result;
use teloxide::prelude::{Bot, Message, Request};
use teloxide::requests::Requester;
use teloxide::types::ChatMemberStatus;

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
