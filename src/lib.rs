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
    if msg.chars().count() > MESSAGE_PREVIEW_SIZE {
        msg = msg
            .chars()
            .take(MESSAGE_PREVIEW_SIZE - 3)
            .collect::<String>();
        msg.push_str("...");
    }
    msg
}

#[cfg(test)]
mod tests {
    use super::*;
    use embeddings::Embeddings;
    use tokio::fs;
    use tokio::io::AsyncReadExt;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_is_spam() {
        let embeddings = Arc::new(Mutex::new(Embeddings::new().await.unwrap()));
        let mut entries = fs::read_dir("test_data").await.unwrap();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            let path = entry.path();
            let mut contents = String::new();
            let mut file = fs::File::open(&path).await.unwrap();
            file.read_to_string(&mut contents).await.unwrap();

            let got = is_spam(&embeddings, &contents).await.unwrap();
            let expected = path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("spam");

            assert_eq!(
                expected,
                got,
                "{} was not flagged as expected",
                path.display(),
            );
        }
    }
}
