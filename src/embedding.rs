use anyhow::{anyhow, Result};
use lru::LruCache;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsConfig, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::num::NonZeroUsize;
use std::os::unix::net::{UnixListener, UnixStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream as TokioUnixStream;

const PATH: &str = ".airnope.embedding.socket";
const EOL: &[u8] = b"\n";

fn to_array(vector: &[f32]) -> Result<[f32; 384]> {
    if vector.len() != 384 {
        return Err(anyhow!("Embedding does not have 384 numbers"));
    }
    let mut result = [0 as f32; 384];
    for (idx, num) in vector.iter().enumerate() {
        result[idx] = *num;
    }
    Ok(result)
}

pub struct ZeroShotClassification {
    model: SentenceEmbeddingsModel,
    cache: LruCache<Vec<u8>, [f32; 384]>,
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

    fn embedding(&mut self, text: String) -> Result<String> {
        let key = text.as_bytes().to_vec();
        let vector = match self.cache.get(&key) {
            Some(v) => *v,
            None => {
                let results = self.model.encode(&[text])?;
                let result = results.first().ok_or(anyhow!("Error creating embedding"))?;
                let embeddings = to_array(result)?;
                self.cache.put(key, embeddings);
                embeddings
            }
        };

        Ok(vector
            .iter()
            .map(|&num| num.to_string())
            .collect::<Vec<String>>()
            .join(","))
    }
}

fn handle_request(model: &mut ZeroShotClassification, mut stream: UnixStream) {
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(size) => {
                let text = String::from_utf8_lossy(&buffer[..size]);
                match model.embedding(text.to_string()) {
                    Err(e) => {
                        log::error!("Error handling embedding request: {}", e);
                        return;
                    }
                    Ok(response) => {
                        if let Err(e) = stream.write_all(response.as_bytes()) {
                            if e.kind() == ErrorKind::BrokenPipe {
                                return;
                            }
                            log::error!("Error writing embedding: {}", e);
                            return;
                        }
                        if let Err(e) = stream.write_all("\n".as_bytes()) {
                            if e.kind() == ErrorKind::BrokenPipe {
                                return;
                            }
                            log::error!("Error sending the EOL for the embedding response: {}", e);
                            return;
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Error handling embedding request: {}", e);
                return;
            }
        }
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
            Ok(stream) => {
                handle_request(&mut model, stream);
            }
            Err(e) => {
                log::error!("Connection to embedding server failed: {}", e);
            }
        }
    }
    Ok(())
}

pub async fn embedding_for(text: &str) -> Result<[f32; 384]> {
    loop {
        if fs::metadata(PATH).is_ok() {
            break;
        }

        log::debug!("Waiting for embedding server...");
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    let mut stream = TokioUnixStream::connect(PATH).await?;
    stream.write_all(text.as_bytes()).await?;

    let mut reader = BufReader::new(&mut stream);
    let mut response = vec![];
    reader.read_until(EOL[0], &mut response).await?;

    let text = String::from_utf8_lossy(&response);
    let result = to_array(
        &text
            .split(',')
            .filter_map(|n| n.trim().parse().ok())
            .collect::<Vec<f32>>(),
    )?;
    Ok(result)
}
