use anyhow::{anyhow, Result};
use moka::future::Cache;
use rust_bert::pipelines::summarization::SummarizationModel;
use std::sync::Arc;
use tokio::{sync::Mutex, task::block_in_place};

pub struct Summarizer {
    model: SummarizationModel,

    cache: Cache<Vec<u8>, String>,
}

impl Summarizer {
    pub async fn new() -> Result<Self> {
        let config = Default::default(); // bert-cnn
        let model = block_in_place(|| SummarizationModel::new(config))?;
        let cache = Cache::new(2_048);
        Ok(Self { model, cache })
    }

    async fn summarize_from_model(&mut self, cache_key: Vec<u8>, text: &str) -> Result<String> {
        let results = self.model.summarize(&[text])?;
        let result = results
            .first()
            .ok_or(anyhow!("Error creating embedding"))?
            .clone();
        self.cache.clone().insert(cache_key, result.clone()).await;
        Ok(result)
    }

    async fn summarize(&mut self, text: &str) -> Result<String> {
        let cache_key = text.as_bytes().to_vec();
        let result = match self.cache.clone().get(&cache_key).await {
            Some(v) => v,
            None => self.summarize_from_model(cache_key, text).await?,
        };
        Ok(result)
    }
}

pub async fn summary_for(model: Arc<Mutex<Summarizer>>, text: &str) -> Result<String> {
    let mut locked = model.lock().await;
    locked.summarize(text).await
}

pub async fn download() -> Result<()> {
    Summarizer::new().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zsc::LABELS;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_summary_for() {
        let model = Arc::new(Mutex::new(Summarizer::new().await.unwrap()));
        let got = summary_for(model, LABELS[0]).await;
        assert!(got.is_ok(), "expected no error, got {:?}", got);
        assert!(got.unwrap() != "");
    }
}
