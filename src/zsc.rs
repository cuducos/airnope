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

    pub async fn score(&self, txt: &str) -> Result<f32> {
        let vector = embedding_for(txt).await?;
        Ok(cosine_distance(vector.to_vec(), self.vector.to_vec()))
    }

    pub async fn is_spam(&self, txt: &str) -> Result<bool> {
        let score = self.score(txt).await?;
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
    use crate::embedding;
    use tokio::io::AsyncReadExt;
    use tokio::runtime::Handle;
    use tokio::{fs, task};

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_is_spam() {
        std::thread::spawn(|| {
            crate::embedding::serve().unwrap();
        });
        embedding::wait_until_ready().await.unwrap();

        let model = task::block_in_place(move || {
            Handle::current().block_on(async move { ZeroShotClassification::new().await.unwrap() })
        });

        let mut entries = fs::read_dir("test_data").await.unwrap();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            let path = entry.path();
            let mut contents = String::new();
            let mut file = fs::File::open(&path).await.unwrap();
            file.read_to_string(&mut contents).await.unwrap();

            let got = model.is_spam(&contents).await.unwrap();
            let expected = path
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("spam");
            let score = model.score(&contents).await.unwrap();

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
