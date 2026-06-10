use anyhow::{anyhow, Context, Result};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{api::sync::Api, Repo};
use moka::future::Cache;
use std::sync::Arc;
use tokenizers::Tokenizer;
use tokio::sync::Mutex;

pub const EMBEDDINGS_SIZE: usize = 384;

pub struct Embeddings {
    model: Option<BertModel>,
    tokenizer: Option<Tokenizer>,
    device: Device,
    cache: Cache<Vec<u8>, [f32; EMBEDDINGS_SIZE]>,
}

impl Embeddings {
    pub async fn new() -> Result<Self> {
        let cache = Cache::new(2_048);
        let device = Device::Cpu;
        Ok(Self {
            model: None,
            tokenizer: None,
            device,
            cache,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        if self.model.is_some() {
            return Ok(());
        }

        log::info!("Loading embedding model...");

        let api = Api::new().context("Failed to create HF API")?;
        let repo = api.repo(Repo::model(
            "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        ));

        let config_filename = repo.get("config.json")?;
        let config: Config = serde_json::from_str(&std::fs::read_to_string(config_filename)?)?;

        let tokenizer_filename = repo.get("tokenizer.json")?;
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(anyhow::Error::msg)?;

        let weights_filename = repo.get("model.safetensors")?;
        let tensors = candle_core::safetensors::load(weights_filename, &self.device)?;
        let vb = VarBuilder::from_tensors(tensors, DTYPE, &self.device);
        let model = BertModel::load(vb, &config)?;

        self.model = Some(model);
        self.tokenizer = Some(tokenizer);
        log::info!("Embedding model loaded and ready.");

        Ok(())
    }

    fn get_model(&mut self) -> Result<(&BertModel, &Tokenizer)> {
        self.init()?;
        let model = self
            .model
            .as_ref()
            .ok_or_else(|| anyhow!("SentenceEmbeddingsModel not initialized after init()"))?;
        let tokenizer = self
            .tokenizer
            .as_ref()
            .ok_or_else(|| anyhow!("Tokenizer not initialized after init()"))?;
        Ok((model, tokenizer))
    }

    async fn calculate_from_model(
        &mut self,
        cache_key: Vec<u8>,
        text: &str,
    ) -> Result<[f32; EMBEDDINGS_SIZE]> {
        let device = self.device.clone();
        let (model, tokenizer) = self.get_model()?;

        let tokens = tokenizer.encode(text, true).map_err(anyhow::Error::msg)?;
        let token_ids = Tensor::new(tokens.get_ids(), &device)?.unsqueeze(0)?;
        let token_type_ids = token_ids.zeros_like()?;

        let embeddings = model.forward(&token_ids, &token_type_ids, None)?;

        let attention_mask = Tensor::new(tokens.get_attention_mask(), &device)?.unsqueeze(0)?;
        let mask = attention_mask
            .unsqueeze(2)?
            .to_dtype(candle_core::DType::F32)?;
        let weighted_embeddings = embeddings.broadcast_mul(&mask)?;
        let summed = weighted_embeddings.sum(1)?;
        let counts = mask.sum(1)?;
        let pooled = summed.broadcast_div(&counts.clamp(1e-9, f32::MAX)?)?;

        let norm = pooled.sqr()?.sum_keepdim(1)?.sqrt()?;
        let normalized = pooled.broadcast_div(&norm)?;

        let vector = normalized.get(0)?.to_vec1::<f32>()?;

        if vector.len() != EMBEDDINGS_SIZE {
            return Err(anyhow!(
                "Embedding does not have {} numbers, has {} instead",
                EMBEDDINGS_SIZE,
                vector.len()
            ));
        }

        let mut result = [0.0f32; EMBEDDINGS_SIZE];
        result.copy_from_slice(&vector);

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
    let mut embeddings = Embeddings::new().await?;
    embeddings.init()?;
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
        assert!(got.is_ok(), "expected no error, got {got:?}");

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
