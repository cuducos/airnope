pub mod common;
pub use common::embeddings;
pub use common::re;
pub use common::telegram;
pub use common::zsc;

use anyhow::Result;
use common::zsc::ZeroShotClassification;
use std::sync::Arc;
use tokio::sync::Mutex;

const MESSAGE_PREVIEW_SIZE: usize = 128;

#[derive(Debug, PartialEq)]
pub struct Guess {
    pub is_spam: bool,
    pub score: Option<f32>,
    pub scores: Vec<f32>,
}

pub async fn is_spam_with_custom_classifier(
    embeddings: &Arc<Mutex<embeddings::Embeddings>>,
    classifier: ZeroShotClassification,
    txt: &str,
) -> Result<Guess> {
    let regex = re::RegularExpression::new().await?;
    let result = regex.is_spam(txt).await?;
    if !result.is_spam {
        return Ok(result);
    }
    classifier.is_spam(embeddings, txt).await
}
pub async fn is_spam(embeddings: &Arc<Mutex<embeddings::Embeddings>>, txt: &str) -> Result<Guess> {
    let zero_shot = zsc::ZeroShotClassification::default(embeddings).await?;
    is_spam_with_custom_classifier(embeddings, zero_shot, txt).await
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
    use tokio::{fs, io::AsyncReadExt};
    use zsc::THRESHOLD;

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
                got.is_spam,
                "{} was not flagged as expected",
                path.display(),
            );
            if expected {
                assert!(
                    got.score.unwrap_or(0.0) > THRESHOLD,
                    "expected score for {} to be greater than {}, got {}",
                    path.display(),
                    THRESHOLD,
                    got.score.unwrap_or(0.0)
                );
            }
        }
    }
}
