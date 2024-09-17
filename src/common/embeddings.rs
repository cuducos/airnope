use anyhow::{anyhow, Result};
use moka::future::Cache;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsConfig, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use std::sync::Arc;
use tokio::{sync::Mutex, task::block_in_place};

pub const EMBEDDINGS_SIZE: usize = 384;

pub struct Embeddings {
    model: SentenceEmbeddingsModel,
    cache: Cache<Vec<u8>, [f32; EMBEDDINGS_SIZE]>,
}

impl Embeddings {
    pub async fn new() -> Result<Self> {
        let config = SentenceEmbeddingsConfig::from(SentenceEmbeddingsModelType::AllMiniLmL6V2);
        let model = block_in_place(|| SentenceEmbeddingsModel::new(config))?;
        let cache = Cache::new(2_048);
        Ok(Self { model, cache })
    }

    async fn calculate_from_model(
        &mut self,
        cache_key: Vec<u8>,
        text: &str,
    ) -> Result<[f32; EMBEDDINGS_SIZE]> {
        let results = self.model.encode(&[text])?;
        let vector = results
            .first()
            .ok_or(anyhow!("Error creating embedding"))?
            .clone();
        if vector.len() != EMBEDDINGS_SIZE {
            return Err(anyhow!(
                "Embedding does not have {} numbers, has {} instead",
                EMBEDDINGS_SIZE,
                vector.len()
            ));
        }
        let mut result = [0 as f32; EMBEDDINGS_SIZE];
        for (idx, num) in vector.iter().enumerate() {
            result[idx] = *num;
        }
        self.cache.clone().insert(cache_key, result).await;
        Ok(result)
    }

    async fn create(&mut self, text: &str) -> Result<[f32; EMBEDDINGS_SIZE]> {
        let cache_key = text.as_bytes().to_vec();
        let result = match self.cache.clone().get(&cache_key).await {
            Some(v) => v,
            None => self.calculate_from_model(cache_key, text).await?,
        };
        Ok(result)
    }
}

pub async fn embeddings_for(
    model: Arc<Mutex<Embeddings>>,
    text: String,
) -> Result<[f32; EMBEDDINGS_SIZE]> {
    let mut locked = model.lock().await;
    locked.create(text.as_str()).await
}

pub async fn download() -> Result<()> {
    Embeddings::new().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zsc::LABELS;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_embeddings_for() {
        let model = Arc::new(Mutex::new(Embeddings::new().await.unwrap()));
        let got = embeddings_for(model, LABELS[0].to_string()).await;
        assert!(got.is_ok(), "expected no error, got {:?}", got);

        let vector = got.unwrap();
        assert_eq!(
            vector.len(),
            EMBEDDINGS_SIZE,
            "expected {}, got {:?}",
            EMBEDDINGS_SIZE,
            vector.len()
        );
        assert!(vector[0] != 0.0);
    }
}
