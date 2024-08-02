use anyhow::Result;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use teloxide::prelude::{Bot, Message, *};
use teloxide::requests::Requester;
use teloxide::respond;
use teloxide::types::{ChatMemberStatus, MessageKind};

const A: &str = "[аa🅰🅰️🇦🇦]";
const I: &str = "[іiI1lℹ️🇮]";
const R: &str = "[рr🇷]";
const D: &str = "[dԁ🇩]";
const O: &str = "[оo0🅾️🇴]";
const P: &str = "[рpρϱ🅿️🇵]";

lazy_static! {
    static ref AIRDROP: Regex = RegexBuilder::new(
        [A, I, R, D, R, O, P]
            .map(|s| s.to_string())
            .join(r"\s?")
            .as_str()
    )
    .case_insensitive(true)
    .build()
    .unwrap();
}

fn is_spam(msg: Option<&str>) -> bool {
    if let Some(txt) = msg {
        let result = AIRDROP.is_match(txt);
        if result {
            log::debug!("Message detected as spam: {:?}", txt);
        }
        return result;
    }
    false
}

async fn delete(bot: &Bot, msg: &Message) -> Result<()> {
    bot.delete_message(msg.chat.id, msg.id).send().await?;
    Ok(())
}

async fn ban(bot: &Bot, msg: &Message) -> Result<()> {
    if let Some(user) = msg.from() {
        bot.kick_chat_member(msg.chat.id, user.id).send().await?;
    }
    Ok(())
}

async fn is_admin(bot: &Bot, msg: &Message) -> bool {
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

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init(); // based on RUST_LOG environment variable
    let bot = Bot::from_env(); // requires TELOXIDE_TOKEN environment variable
    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        if let MessageKind::Common(_) = &msg.kind {
            if is_spam(msg.text()) && !is_admin(&bot, &msg).await {
                if let Err(e) = tokio::try_join!(delete(&bot, &msg), ban(&bot, &msg)) {
                    log::error!("Error handling spam: {:?}", e);
                }
            }
        }
        respond(())
    })
    .await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_spam() {
        let test_cases = vec![
            ("airdrop", true),
            ("аirdrop", true), // Cyrillic а
            ("аirdrор", true), // Cyrillic а and Cyrillic о
            ("аirdrоp", true), // Cyrillic а and Cyrillic о
            ("аirdrор", true), // Cyrillic а, Cyrillic о, and Cyrillic р
            ("airdrор", true), // Cyrillic о
            ("airdrоp", true), // Cyrillic о
            ("airdrор", true), // Cyrillic о and Cyrillic р
            ("аirdrоp", true), // Cyrillic а, and Cyrillic о
            ("аirdrор", true), // Cyrillic а, Cyrillic о, and Cyrillic р
            ("аirdrop", true), // Cyrillic а
            ("аirdrор", true), // Cyrillic а and Cyrillic р
            ("аirdrop", true), // Cyrillic а
            ("airԁrop", true), // Cyrillic ԁ
            ("aіrԁrop", true), // Cyrillic і and Cyrillic ԁ
            ("airԁroр", true), // Cyrillic р
            ("аirdrор", true), // Cyrillic а and Cyrillic р
            ("aіrdrop", true), // Cyrillic і
            ("аіrdrop", true), // Cyrillic а and Cyrillic і
            ("аіrdrop", true), // Cyrillic а and Cyrillic і
            ("аіrdrop", true), // Cyrillic а and Cyrillic і
            ("aіrdroр", true), // Cyrillic і and Cyrillic р
            ("аіrdroр", true), // Cyrillic а, Cyrillic і, and Cyrillic р
            ("аirdrор", true), // Cyrillic а and Cyrillic р
            ("aіrԁrор", true), // Cyrillic і, Cyrillic ԁ, and Cyrillic р
            ("aіrԁrop", true), // Cyrillic і and Cyrillic ԁ
            ("airdroр", true), // Cyrillic р
            ("airԁrop", true), // Greek delta, Δ
            ("аirdrор", true), // Greek o, ο
            ("аіrԁrop", true), // Greek iota, ι
            ("airԁroр", true), // Greek rho, ρ
            ("аirdrор", true), // Greek omicron, ο
            ("aіrdrop", true), // Greek iota, ι
            ("аіrdrop", true), // Greek alpha, α
            ("аіrdrop", true), // Greek iota, ι
            ("aіrdroр", true), // Greek iota, ι, and rho, ρ
            ("аirdrор", true), // Greek omicron, ο, and rho, ρ
            ("aіrԁrор", true), // Greek iota, ι, delta, Δ, and rho, ρ
            ("aіrԁrop", true), // Greek iota, ι, and delta, Δ
            ("airdroр", true), // Greek rho, ρ
            ("Сlаim  Q СOMMUNITY АIRDROP\n Join the Q movement.", true), // snippet from a real one
            ("🅰irdrop", true), // with emoji
            ("🅰️ℹ️irdr🅾️🇵", true), // with emojis
            ("air drop", true), // with space
            ("a i r d r o p", true), // with single spaces
            ("a i r d r o p", true), // with different kids of spaces
            ("🇦 🇮 🇷 🇩 🇷 🇴 🇵", true), // with special characters and spaces
            ("42", false),
            ("", false),
        ];
        for (word, expected) in test_cases {
            for w in [word, word.to_uppercase().as_str()] {
                let got = is_spam(Some(w));
                assert_eq!(
                    got, expected,
                    "expected: {:?} for {:?}, got: {:?}",
                    expected, w, got
                );
            }
        }
        assert!(!is_spam(None));
    }
}
