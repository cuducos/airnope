use anyhow::{anyhow, Result};
use lru::LruCache;
use rmp_serde::Deserializer;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsConfig, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::num::NonZeroUsize;
use std::os::unix::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream as TokioUnixStream;

const PATH: &str = ".airnope.embedding.socket";

#[derive(Deserialize, Serialize)]
pub struct Request {
    text: String,
}

impl Request {
    fn cache_key(&self) -> Vec<u8> {
        self.text.as_bytes().to_vec()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Response {
    vector: Vec<f32>,
}

impl Response {
    fn to_array(&self) -> Result<[f32; 384]> {
        if self.vector.len() != 384 {
            return Err(anyhow!("Embedding does not have 384 numbers"));
        }
        let mut result = [0 as f32; 384];
        for (idx, num) in self.vector.iter().enumerate() {
            result[idx] = *num;
        }
        Ok(result)
    }
}

pub struct ZeroShotClassification {
    model: SentenceEmbeddingsModel,
    cache: LruCache<Vec<u8>, Response>,
}

impl ZeroShotClassification {
    pub fn new() -> Result<Self> {
        let config = SentenceEmbeddingsConfig::from(SentenceEmbeddingsModelType::AllMiniLmL6V2);
        let model = SentenceEmbeddingsModel::new(config)?;
        let cache = LruCache::new(
            NonZeroUsize::new(1024).ok_or(anyhow!("Could not create LRU cache size"))?,
        );
        Ok(Self { model, cache })
    }

    pub fn embedding(&mut self, request: Request) -> Result<Response> {
        let key = request.cache_key();
        match self.cache.get(&key) {
            Some(response) => Ok(response.clone()),
            None => {
                let results = self.model.encode(&[request.text])?;
                let vector = results.first().ok_or(anyhow!("Error creating embedding"))?;
                let response = Response {
                    vector: vector.clone(),
                };
                self.cache.put(key, response.clone());
                Ok(response)
            }
        }
    }
}

async fn async_bytes_from_stream(mut stream: TokioUnixStream) -> Result<Vec<u8>> {
    let mut prefix = [0; 8]; // prefix has the size of the message
    stream.read_exact(&mut prefix).await?;
    let size = usize::from_be_bytes(prefix);
    let mut buf: Vec<u8> = vec![0; size];
    stream.read_exact(&mut buf).await?;
    Ok(buf)
}

fn bytes_from_stream(stream: &mut UnixStream) -> Result<Vec<u8>> {
    let mut prefix = [0; 8]; // prefix has the size of the message
    stream.read_exact(&mut prefix)?;
    let size = usize::from_be_bytes(prefix);
    let mut buf: Vec<u8> = vec![0; size];
    stream.read_exact(&mut buf)?;
    Ok(buf)
}

fn handle_request_body(model: &mut ZeroShotClassification, stream: &mut UnixStream) -> Result<()> {
    let body = bytes_from_stream(stream)?;
    let mut deserializer = Deserializer::new(&body[..]);
    let request: Request = Deserialize::deserialize(&mut deserializer)?;
    let response = rmp_serde::to_vec(&model.embedding(request)?)?;
    let size = response.len().to_be_bytes();
    stream.write_all(&size)?; // size as message prefix
    stream.write_all(&response)?;
    Ok(())
}

fn handle_request(model: &mut ZeroShotClassification, stream: &mut UnixStream) {
    if let Err(e) = handle_request_body(model, stream) {
        log::error!("Error handling embedding request: {}", e);
    }
}

pub fn serve() -> Result<()> {
    if fs::metadata(PATH).is_ok() {
        fs::remove_file(PATH)?;
    }

    let mut model = ZeroShotClassification::new()?;
    let listener = UnixListener::bind(PATH)?;
    log::info!("Embedding server listening on {}", PATH);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_request(&mut model, &mut stream);
            }
            Err(e) => {
                log::error!("Connection to embedding server failed: {}", e);
            }
        }
    }
    Ok(())
}

pub async fn wait_until_ready() -> Result<()> {
    loop {
        if fs::metadata(PATH).is_ok() {
            break;
        }

        log::debug!("Waiting for embedding socket...");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    let mut retries = 32;
    let mut stream = TokioUnixStream::connect(PATH).await;
    while retries > 0 {
        if stream.is_ok() {
            return Ok(());
        }

        log::debug!("Waiting for connection to the embedding socket...");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        stream = TokioUnixStream::connect(PATH).await;
        retries -= 1;
    }

    Err(anyhow!(
        "Could not connect to the embedding socket: {:?}",
        stream
    ))
}

pub async fn embeddings_for(text: &str) -> Result<[f32; 384]> {
    let mut stream = TokioUnixStream::connect(PATH).await?;
    let request = rmp_serde::to_vec(&Request {
        text: text.to_string(),
    })?;
    let size = request.len().to_be_bytes(); // size as message prefix
    stream.write_all(&size).await?;
    stream.write_all(&request).await?;
    let buf = async_bytes_from_stream(stream).await?;
    let mut deserializer = Deserializer::new(&buf[..]);
    let response: Response = Deserialize::deserialize(&mut deserializer)?;
    response.to_array()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_embeddings_for() {
        std::thread::spawn(|| {
            serve().unwrap();
        });
        wait_until_ready().await.unwrap();

        let got = embeddings_for("umbrella").await;
        assert!(got.is_ok(), "expected no error, got {:?}", got);
        assert!(got.unwrap()[0] != 0.0);
    }
}
