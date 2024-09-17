use crate::{
    embeddings::{embeddings_for, Embeddings, EMBEDDINGS_SIZE},
    truncated, Guess,
};
use acap::cos::cosine_distance;
use anyhow::Result;
use futures::future::try_join_all;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::sync::Arc;
use tokio::sync::Mutex;

pub const LABELS: [&str; 3] = [
    "claim crypto airdrop spam",
    "airdrop event announcement",
    "investment opportunity",
];
pub const THRESHOLD: f32 = 0.5;

type LabelVectors = Vec<[f32; EMBEDDINGS_SIZE]>;

#[derive(Clone)]
pub struct ZeroShotClassification {
    vectors: LabelVectors,
}

pub fn average_without_extremes(scores: &Vec<f32>) -> f32 {
    if scores.is_empty() {
        return 0.0;
    }
    if scores.len() < 3 {
        return scores.iter().sum::<f32>() / scores.len() as f32;
    }
    let mut sum = 0.0;
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    for &score in scores {
        if score < min {
            min = score;
        }
        if score > max {
            max = score;
        }
        sum += score;
    }
    (sum - (min + max)) / (scores.len() as f32 - 2.0)
}

impl ZeroShotClassification {
    pub async fn new<T>(embeddings: &Arc<Mutex<Embeddings>>, labels: T) -> Result<Self>
    where
        T: IntoIterator,
        T::Item: AsRef<str>,
    {
        let vectors: Vec<[f32; EMBEDDINGS_SIZE]> = try_join_all(
            labels
                .into_iter()
                .map(|label| embeddings_for(Arc::clone(embeddings), label.as_ref().to_string())),
        )
        .await?;

        Ok(Self { vectors })
    }

    pub async fn default(embeddings: &Arc<Mutex<Embeddings>>) -> Result<Self> {
        Self::new(embeddings, LABELS).await
    }

    pub async fn is_spam(&self, embeddings: &Arc<Mutex<Embeddings>>, txt: &str) -> Result<Guess> {
        let vector = embeddings_for(Arc::clone(embeddings), txt.to_string()).await?;
        let scores = self
            .vectors
            .par_iter()
            .map(|label| cosine_distance(label.to_vec(), vector.to_vec()))
            .collect::<Vec<f32>>();
        let score = average_without_extremes(&scores);
        let result = score > THRESHOLD;
        if result {
            log::info!(
                "Message detected as spam by ZeroShotClassification (score = {})",
                score,
            );
            log::debug!("{}", truncated(txt));
        }
        Ok(Guess {
            is_spam: result,
            score: Some(score),
            scores,
        })
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
        let model = ZeroShotClassification::default(&embeddings).await.unwrap();

        let mut entries = fs::read_dir("test_data").await.unwrap();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            let path = entry.path();
            let mut contents = String::new();
            let mut file = fs::File::open(&path).await.unwrap();
            file.read_to_string(&mut contents).await.unwrap();

            let got = model.is_spam(&embeddings, &contents).await.unwrap();
            if let Some(score) = got.score {
                let expected = path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("spam");

                assert_eq!(
                    expected,
                    got.is_spam,
                    "{} was not flagged as expected (score = {}, threshold = {})",
                    path.display(),
                    score,
                    THRESHOLD,
                );
            } else {
                panic!("{} got no score", path.display());
            }
        }
    }

    #[test]
    fn test_average_without_extremes() {
        let scores = vec![1.0, 4.0, 6.0, 9.0];
        assert_eq!(average_without_extremes(&scores), 5.0);
    }
}
