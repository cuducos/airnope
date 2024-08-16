use acap::cos::cosine_distance;
use airnope::{
    embeddings::{embeddings_for, Embeddings, EMBEDDINGS_SIZE},
    zsc::{LABELS, THRESHOLD},
};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use futures::future::try_join_all;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Clone)]
struct Input {
    labels: Vec<String>,
    vectors: Vec<[f32; EMBEDDINGS_SIZE]>,
}

impl Input {
    async fn new(embeddings: &Arc<Mutex<Embeddings>>, labels: Vec<String>) -> Result<Self> {
        let vectors = try_join_all(
            labels
                .iter()
                .map(|txt| embeddings_for(embeddings.clone(), txt.as_str()))
                .collect::<Vec<_>>(),
        )
        .await?;
        Ok(Self { labels, vectors })
    }

    fn to_string(&self, idx: usize) -> String {
        let prefix = if idx == 0 {
            "Reference".to_string()
        } else {
            format!("Alternative {}", idx)
        };
        let base = format!("\n==> {}: {}", prefix, self.labels.join(" + "));
        if idx == 0 {
            format!("{} (threshold: {:.2})", base, THRESHOLD)
        } else {
            base
        }
    }
}

struct Task {
    name: String,
    is_spam: bool,
    content: String,
}

impl Task {
    fn new(path: &PathBuf) -> Result<Self> {
        let name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => return Err(anyhow!("Could not get file name")),
        };
        let is_spam = name.starts_with("spam");
        Ok(Self {
            name,
            is_spam,
            content: fs::read_to_string(path).context(format!("Reading {}", path.display()))?,
        })
    }
}

struct Evaluation {
    scores: Vec<f32>,
    score: f32,
    expected: bool,
}

impl Evaluation {
    async fn new(embeddings: &Arc<Mutex<Embeddings>>, task: &Task, input: Input) -> Result<Self> {
        let message = embeddings_for(embeddings.clone(), task.content.as_str()).await?;
        let scores: Vec<f32> = input
            .vectors
            .into_par_iter()
            .map(|label| cosine_distance(label.to_vec(), message.to_vec()))
            .collect();

        let score = scores.iter().sum::<f32>() / scores.len() as f32;
        let is_spam = score > THRESHOLD;
        let expected = task.is_spam == is_spam;
        Ok(Self {
            scores,
            score,
            expected,
        })
    }

    fn to_string(&self, task: &Task) -> String {
        let mark = if self.expected { "✔" } else { "✘" };
        let diff = self.score - THRESHOLD;
        let prefix = if diff > 0.0 {
            "+"
        } else if diff < 0.0 {
            ""
        } else {
            " "
        };
        let mut output = format!(
            "    {} {: <13} {:.3} ({}{:.3})",
            mark, task.name, self.score, prefix, diff
        );
        if self.scores.len() > 1 {
            output = format!(
                "{} {}",
                output,
                self.scores
                    .iter()
                    .map(|&score| format!("{:.3}", score))
                    .collect::<Vec<String>>()
                    .join(" | ")
            );
        }
        output
    }
}

fn paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let files = fs::read_dir(Path::new("test_data"))?;
    for file in files {
        let path = file?.path();
        if path.is_file() {
            paths.push(path);
        }
    }
    paths.sort_by(|a, b| {
        a.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .cmp(&b.file_name().unwrap_or_default().to_string_lossy())
    });
    Ok(paths)
}

fn labels() -> Result<Vec<Vec<String>>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(anyhow!("Usage: airnope-bench <label1> [label2] ...",));
    }
    let mut labels = vec![LABELS.into_iter().map(|label| label.to_string()).collect()];
    labels.extend(
        args[1..]
            .iter()
            .map(|label| label.split(',').map(|val| val.trim().to_string()).collect())
            .collect::<Vec<_>>(),
    );
    Ok(labels.to_vec())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    pretty_env_logger::init(); // based on RUST_LOG environment variable
    let labels = labels()?;
    let paths = paths()?;
    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?)).clone();
    for (idx, label) in labels.iter().enumerate() {
        let input = Input::new(&embeddings, label.clone()).await?;
        println!("{}", input.to_string(idx).blue().bold());
        let mut scores: Vec<f32> = vec![];
        for path in paths.iter() {
            let task = Task::new(path)?;
            let evaluation = Evaluation::new(&embeddings, &task, input.clone()).await?;
            let line = evaluation.to_string(&task);
            println!(
                "{}",
                if evaluation.expected {
                    line.green()
                } else {
                    line.red()
                }
            );
            scores.push(evaluation.score);
        }
    }
    Ok(())
}
