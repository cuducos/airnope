pub mod common;
pub use common::embeddings;
pub use common::re;
pub use common::summary;
pub use common::telegram;
pub use common::zsc;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

const MESSAGE_PREVIEW_SIZE: usize = 128;

pub async fn is_spam(embeddings: &Arc<Mutex<embeddings::Embeddings>>, txt: &str) -> Result<bool> {
    let regex = re::RegularExpression::new().await?;
    if !regex.is_spam(txt).await? {
        return Ok(false);
    }
    let zero_shot = zsc::ZeroShotClassification::new(embeddings).await?;
    zero_shot.is_spam(embeddings, txt).await
}

fn truncated(message: &str) -> String {
    let mut msg = message.to_string();
    msg.retain(|c| !c.is_control() || c == ' ');
    msg = msg.trim().to_string();
    if msg.len() > MESSAGE_PREVIEW_SIZE {
        msg = msg[..(MESSAGE_PREVIEW_SIZE - 3)].to_string();
        msg.push_str("...");
    }
    msg
}
