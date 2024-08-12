use std::sync::Arc;

use crate::embeddings::{embeddings_for, Embeddings};
use acap::cos::cosine_distance;
use anyhow::Result;
use tokio::sync::Mutex;

pub const LABEL: &str = "claim crypto airdrop spam";
pub const THRESHOLD: f32 = 0.5;

#[derive(Clone)]
pub struct ZeroShotClassification {
    vector: [f32; 384],
}

impl ZeroShotClassification {
    pub async fn new(embeddings: &Arc<Mutex<Embeddings>>) -> Result<Self> {
        let vector = embeddings_for(Arc::clone(embeddings), LABEL).await?;
        Ok(Self { vector })
    }

    pub async fn score(&self, embeddings: &Arc<Mutex<Embeddings>>, txt: &str) -> Result<f32> {
        let vector = embeddings_for(Arc::clone(embeddings), txt).await?;
        Ok(cosine_distance(vector.to_vec(), self.vector.to_vec()))
    }

    pub async fn is_spam(&self, embeddings: &Arc<Mutex<Embeddings>>, txt: &str) -> Result<bool> {
        let score = self.score(embeddings, txt).await?;
        let result = score > THRESHOLD;
        if result {
            log::debug!(
                "Message detected as spam by ZeroShotClassification (score = {:?}): {:?}",
                score,
                txt,
            );
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;
    use tokio::io::AsyncReadExt;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_is_spam() {
        let embeddings = Arc::new(Mutex::new(Embeddings::new().await.unwrap()));
        let model = ZeroShotClassification::new(&embeddings).await.unwrap();

        let mut entries = fs::read_dir("test_data").await.unwrap();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            let path = entry.path();
            let mut contents = String::new();
            let mut file = fs::File::open(&path).await.unwrap();
            file.read_to_string(&mut contents).await.unwrap();

            let got = model.is_spam(&embeddings, &contents).await.unwrap();
            let expected = path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("spam");
            let score = model.score(&embeddings, &contents).await.unwrap();

            assert_eq!(
                expected,
                got,
                "{} was not flagged as expected (score = {}, threshold = {})",
                path.display(),
                score,
                THRESHOLD,
            );
        }
    }
}
