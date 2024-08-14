pub mod common;
pub use common::embeddings;
pub use common::re;
pub use common::telegram;
pub use common::zsc;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn is_spam(embeddings: &Arc<Mutex<embeddings::Embeddings>>, txt: &str) -> Result<bool> {
    let regex = re::RegularExpression::new().await?;
    if !regex.is_spam(txt).await? {
        return Ok(false);
    }
    let zero_shot = zsc::ZeroShotClassification::new(embeddings).await?;
    zero_shot.is_spam(embeddings, txt).await
}
