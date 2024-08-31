use crate::{
    embeddings::{embeddings_for, Embeddings, EMBEDDINGS_SIZE},
    truncated,
};
use acap::cos::cosine_distance;
use anyhow::Result;
use futures::future::try_join_all;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::sync::Arc;
use tokio::sync::Mutex;

pub const LABELS: [&str; 3] = [
    "claim crypto airdrop spam",
    "airdrop event announcement",
    "investment opportunity",
];
pub const THRESHOLD: f32 = 0.5;

type LabelVectors = [[f32; EMBEDDINGS_SIZE]; LABELS.len()];

#[derive(Clone)]
pub struct ZeroShotClassification {
    vectors: LabelVectors,
}

pub fn average_without_extremes(scores: &Vec<f32>) -> f32 {
    if scores.len() < 3 {
        return scores.iter().sum::<f32>() / scores.len() as f32;
    }
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    for score in scores {
        if *score < min {
            min = *score;
        }
        if *score > max {
            max = *score;
        }
    }
    scores.iter().sum::<f32>() - (min + max) / (scores.len() - 2) as f32
}

impl ZeroShotClassification {
    pub async fn new(embeddings: &Arc<Mutex<Embeddings>>) -> Result<Self> {
        match try_join_all(
            LABELS
                .iter()
                .map(|label| embeddings_for(Arc::clone(embeddings), label)),
        )
        .await?
        .try_into()
        {
            Ok(vectors) => Ok(Self { vectors }),
            Err(_) => Err(anyhow::anyhow!("Failed to get embeddings for labels")),
        }
    }

    pub async fn score(&self, embeddings: &Arc<Mutex<Embeddings>>, txt: &str) -> Result<f32> {
        let vector = embeddings_for(Arc::clone(embeddings), txt).await?;
        let scores = self
            .vectors
            .into_par_iter()
            .map(|label| cosine_distance(label.to_vec(), vector.to_vec()))
            .collect::<Vec<f32>>();
        Ok(average_without_extremes(&scores))
    }

    pub async fn is_spam(&self, embeddings: &Arc<Mutex<Embeddings>>, txt: &str) -> Result<bool> {
        let score = self.score(embeddings, txt).await?;
        let result = score > THRESHOLD;
        if result {
            log::info!(
                "Message detected as spam by ZeroShotClassification (score = {})",
                score,
            );
            log::debug!("{}", truncated(txt));
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
