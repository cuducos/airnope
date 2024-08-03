use anyhow::{anyhow, Result};
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsConfig, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream as TokioUnixStream;

const PATH: &str = ".airnope.embedding.socket";
const EOL: &[u8] = b"\n";

pub struct ZeroShotClassification {
    model: SentenceEmbeddingsModel,
}

impl ZeroShotClassification {
    pub fn new() -> Result<Self> {
        let config = SentenceEmbeddingsConfig::from(SentenceEmbeddingsModelType::AllMiniLmL6V2);
        let model = SentenceEmbeddingsModel::new(config)?;
        Ok(Self { model })
    }

    fn embedding(&self, text: String) -> Result<String> {
        let embeddings = self.model.encode(&[text])?;
        let vector = embeddings
            .first()
            .ok_or(anyhow!("Error creating embedding"))?;

        Ok(vector
            .iter()
            .map(|&num| num.to_string())
            .collect::<Vec<String>>()
            .join(","))
    }
}

fn handle_request(model: &ZeroShotClassification, mut stream: UnixStream) {
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

    let model = ZeroShotClassification::new()?;
    let listener = UnixListener::bind(PATH)?;
    log::info!("Embedding server listening on {}", PATH);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_request(&model, stream);
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
    let mut result = [0 as f32; 384];
    for (idx, num) in text.split(',').enumerate() {
        if idx >= 384 {
            return Err(anyhow!("Embedding got more than 384 numbers"));
        }
        match num.trim().parse::<f32>() {
            Ok(n) => result[idx] = n,
            Err(e) => return Err(anyhow!("Cannot convert {:?} to `f32`: {}", num, e)),
        }
    }
    Ok(result)
}
