use crate::embeddings::{embeddings_for, Embeddings, EMBEDDINGS_SIZE};
use crate::zsc::{LABEL, THRESHOLD};
use acap::cos::cosine_distance;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

struct Input {
    label: String,
    vector: [f32; EMBEDDINGS_SIZE],
}

impl Input {
    async fn new(embeddings: &Arc<Mutex<Embeddings>>, text: &str) -> Result<Self> {
        Ok(Self {
            label: text.to_string(),
            vector: embeddings_for(embeddings.clone(), text).await?,
        })
    }

    fn to_string(&self, idx: usize) -> String {
        let prefix = if idx == 0 {
            "Reference".to_string()
        } else {
            format!("Alternative {}", idx)
        };
        let base = format!("\n==> {}: {}", prefix, self.label);

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
    score: f32,
    expected: bool,
}

impl Evaluation {
    async fn new(embeddings: &Arc<Mutex<Embeddings>>, task: &Task, input: &Input) -> Result<Self> {
        let vector = embeddings_for(embeddings.clone(), task.content.as_str()).await?;
        let score = cosine_distance(vector.to_vec(), input.vector.to_vec());
        let is_spam = score > THRESHOLD;
        let expected = task.is_spam == is_spam;
        Ok(Self { score, expected })
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
        format!(
            "    {} {: <13} {:.3} ({}{:.3})",
            mark, task.name, self.score, prefix, diff
        )
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

fn labels() -> Result<Vec<String>> {
    let mut args: Vec<String> = env::args().collect();
    let start = match args.iter().position(|arg| arg.as_str() == "--bench") {
        Some(idx) => idx,
        None => {
            return Err(anyhow!("Could not find --bench flag"));
        }
    };
    args[start] = LABEL.to_string();
    let labels = &args[start..];
    if labels.len() < 2 {
        return Err(anyhow!("Usage: airnope --bench <label1> <label2> ...",));
    }
    Ok(labels.to_vec())
}

pub async fn run() -> Result<()> {
    let labels = labels()?;
    let paths = paths()?;
    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?)).clone();
    for (idx, label) in labels.iter().enumerate() {
        let input = Input::new(&embeddings, label).await?;
        println!("{}", input.to_string(idx).blue().bold());
        let mut scores: Vec<f32> = vec![];
        for path in paths.iter() {
            let task = Task::new(path)?;
            let evaluation = Evaluation::new(&embeddings, &task, &input).await?;
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
