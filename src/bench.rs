use airnope::{
    embeddings::Embeddings,
    is_spam_with_custom_classifier,
    zsc::{ZeroShotClassification, LABELS, THRESHOLD},
};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Clone)]
struct Input {
    classifier: ZeroShotClassification,
    labels: Vec<String>,
    not_spam_scores: Vec<f32>,
    spam_scores: Vec<f32>,
}

impl Input {
    async fn new(embeddings: &Arc<Mutex<Embeddings>>, labels: Vec<String>) -> Result<Self> {
        let classifier = ZeroShotClassification::new(embeddings, labels.clone()).await?;
        Ok(Self {
            classifier,
            labels,
            not_spam_scores: vec![],
            spam_scores: vec![],
        })
    }

    fn to_string(&self, idx: usize) -> String {
        let prefix = format!("Alternative {}", idx + 1);
        let base = format!("\n==> {}: {}", prefix, self.labels.join(" + "));
        if idx == 0 {
            format!("{base} (threshold: {THRESHOLD:.2})")
        } else {
            base
        }
    }

    fn push(&mut self, path: &Path, score: f32) -> Result<()> {
        let name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => return Err(anyhow!("Could not get file name")),
        };
        if name.starts_with("spam") {
            self.spam_scores.push(score);
        } else {
            self.not_spam_scores.push(score);
        }
        Ok(())
    }

    fn stats(&self) {
        let not_spam = self
            .not_spam_scores
            .iter()
            .fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let spam = self
            .spam_scores
            .iter()
            .fold(f32::INFINITY, |a, &b| a.min(b));
        let output = if not_spam > spam {
            format!(
                "\n     No possible threshold (maximum not spam = {not_spam:.3} and minimum spam = {spam:.3})"
            )
            .bold()
            .yellow()
        } else {
            format!("\n     Possible threshold between {not_spam:.3} and {spam:.3}")
                .bold()
                .green()
        };
        println!("{output}");
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
        let result =
            is_spam_with_custom_classifier(embeddings, input.classifier, task.content.as_str())
                .await?;
        let expected = task.is_spam == result.is_spam;
        let score = result.score.unwrap_or(0.0);
        let scores = result.scores;
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
                    .map(|&score| format!("{score:.3}"))
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

fn labels(args: Option<Vec<String>>) -> Vec<Vec<String>> {
    match args {
        None => vec![LABELS.into_iter().map(|label| label.to_string()).collect()],
        Some(labels) => labels
            .iter()
            .map(|label| label.split(',').map(|val| val.trim().to_string()).collect())
            .collect::<Vec<_>>(),
    }
}

async fn simulate(embeddings: Arc<Mutex<Embeddings>>, input: Input, path: &PathBuf) -> Result<f32> {
    let task = Task::new(path)?;
    let evaluation = Evaluation::new(&embeddings, &task, input.clone()).await?;
    let line = evaluation.to_string(&task);
    println!(
        "{}",
        if evaluation.expected {
            line.green()
        } else {
            line.red()
        },
    );
    Ok(evaluation.score)
}

pub async fn run(args: Option<Vec<String>>, pattern: Option<String>) -> Result<()> {
    let regex = pattern
        .map(|pattern| regex::Regex::new(&pattern))
        .transpose()?;
    let labels = labels(args);
    let paths = paths()?;
    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?)).clone();
    for (idx, label) in labels.iter().enumerate() {
        let mut input = Input::new(&embeddings, label.clone()).await?;
        println!("{}", input.to_string(idx).blue().bold());
        for path in paths.iter() {
            if let Some(r) = &regex {
                if !r.is_match(path.to_string_lossy().as_ref()) {
                    continue;
                }
            }
            input.push(
                path,
                simulate(embeddings.clone(), input.clone(), path).await?,
            )?;
        }
        input.stats()
    }
    Ok(())
}
