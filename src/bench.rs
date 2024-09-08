use acap::cos::cosine_distance;
use airnope::{
    common::summary::{summary_for, Summarizer},
    embeddings::{embeddings_for, Embeddings, EMBEDDINGS_SIZE},
    zsc::{average_without_extremes, LABELS, THRESHOLD},
};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use futures::future::try_join_all;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Clone)]
struct Input {
    labels: Vec<String>,
    vectors: Vec<[f32; EMBEDDINGS_SIZE]>,
    not_spam_scores: Vec<f32>,
    spam_scores: Vec<f32>,
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
        Ok(Self {
            labels,
            vectors,
            not_spam_scores: vec![],
            spam_scores: vec![],
        })
    }

    fn to_string(&self, idx: usize) -> String {
        let prefix = format!("Alternative {}", idx+1);
        let base = format!("\n==> {}: {}", prefix, self.labels.join(" + "));
        if idx == 0 {
            format!("{} (threshold: {:.2})", base, THRESHOLD)
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
                "\n     No possible threshold (maximum not spam = {:.3} and minimum spam = {:.3})",
                not_spam, spam
            )
            .bold()
            .yellow()
        } else {
            format!(
                "\n     Possible threshold between {:.3} and {:.3}",
                not_spam, spam
            )
            .bold()
            .green()
        };
        println!("{}", output);
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
    async fn new(
        embeddings: &Arc<Mutex<Embeddings>>,
        summarizer: Option<&Arc<Mutex<Summarizer>>>,
        task: &Task,
        input: Input,
    ) -> Result<Self> {
        let text = match summarizer {
            Some(model) => summary_for(model.clone(), task.content.as_str()).await?,
            None => task.content.clone(),
        };
        let embeddings = embeddings_for(embeddings.clone(), text.as_str()).await?;
        let scores: Vec<f32> = input
            .vectors
            .into_par_iter()
            .map(|label| cosine_distance(label.to_vec(), embeddings.to_vec()))
            .collect();

        let score = average_without_extremes(&scores);
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

fn labels(args: Option<Vec<String>>) -> Vec<Vec<String>> {
    match args {
        None => vec![LABELS.into_iter().map(|label| label.to_string()).collect()],
        Some(labels) => labels
            .iter()
            .map(|label| label.split(',').map(|val| val.trim().to_string()).collect())
            .collect::<Vec<_>>(),
    }
}

async fn simulate(
    embeddings: Arc<Mutex<Embeddings>>,
    sumamrizer: Arc<Mutex<Summarizer>>,
    skip_summary: bool,
    threshold_difference: f32,
    input: Input,
    path: &PathBuf,
) -> Result<f32> {
    let task = Task::new(path)?;
    let mut summarized = false;
    let mut score_without_summarizing = 0.0;
    let mut evaluation = Evaluation::new(&embeddings, None, &task, input.clone()).await?;
    if !skip_summary
        && THRESHOLD - threshold_difference < evaluation.score
        && evaluation.score < THRESHOLD + threshold_difference
    {
        score_without_summarizing = evaluation.score;
        evaluation = Evaluation::new(&embeddings, Some(&sumamrizer), &task, input.clone()).await?;
        summarized = true;
    }
    let line = evaluation.to_string(&task);
    println!(
        "{}{}",
        if evaluation.expected {
            line.green()
        } else {
            line.red()
        },
        if summarized {
            let dif = evaluation.score - score_without_summarizing;
            let sig = if dif > 0.0 {
                "+"
            } else if dif < 0.0 {
                ""
            } else {
                " "
            };
            format!(" ({}{:.2} after summarizing)", sig, dif)
        } else {
            "".to_string()
        }
    );
    Ok(evaluation.score)
}

pub async fn run(
    args: Option<Vec<String>>,
    skip_summary: bool,
    threshold_difference: f32,
) -> Result<()> {
    let labels = labels(args);
    let paths = paths()?;
    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?)).clone();
    let summarizer = Arc::new(Mutex::new(Summarizer::new().await?)).clone();
    for (idx, label) in labels.iter().enumerate() {
        let mut input = Input::new(&embeddings, label.clone()).await?;
        println!("{}", input.to_string(idx).blue().bold());
        for path in paths.iter() {
            input.push(
                path,
                simulate(
                    embeddings.clone(),
                    summarizer.clone(),
                    skip_summary,
                    threshold_difference,
                    input.clone(),
                    path,
                )
                .await?,
            )?;
        }
        input.stats()
    }
    Ok(())
}
