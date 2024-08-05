use crate::embedding::embedding_for;
use acap::cos::cosine_distance;
use anyhow::Result;

const LABEL: &str = "crypto airdrop spam message";
const THRESHOLD: f32 = 0.6;

#[derive(Clone)]
pub struct ZeroShotClassification {
    vector: [f32; 384],
}

impl ZeroShotClassification {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            vector: embedding_for(LABEL).await?,
        })
    }

    pub async fn is_spam(&self, txt: &str) -> Result<bool> {
        let vector = embedding_for(txt).await?;
        let score = cosine_distance(vector.to_vec(), self.vector.to_vec());
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
