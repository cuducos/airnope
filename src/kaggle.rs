use airnope::{
    embeddings::Embeddings, is_spam_with_custom_classifier, zsc::ZeroShotClassification,
};
use anyhow::{anyhow, Context, Result};
use std::{env, io::Cursor, sync::Arc};
use tokio::sync::Mutex;

fn credentials() -> Result<(String, String)> {
    let username =
        env::var("KAGGLE_USERNAME").context("KAGGLE_USERNAME environment variable not set")?;
    let token =
        env::var("KAGGLE_API_TOKEN").context("KAGGLE_API_TOKEN environment variable not set")?;
    if token.trim().is_empty() {
        return Err(anyhow!("KAGGLE_API_TOKEN is set but empty"));
    }
    Ok((username, token))
}

async fn download_dataset(owner: &str, dataset: &str) -> Result<Vec<u8>> {
    let (username, token) = credentials()?;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("Failed to build HTTP client")?;

    let url = format!("https://www.kaggle.com/api/v1/datasets/download/{owner}/{dataset}");
    let response = client
        .get(&url)
        .basic_auth(&username, Some(&token))
        .header("User-Agent", "airnope")
        .send()
        .await
        .context("Failed to download dataset from Kaggle")?;
    let status = response.status();
    if status.is_redirection() {
        let location = response
            .headers()
            .get("location")
            .ok_or_else(|| anyhow!("Kaggle API redirect without location header"))?
            .to_str()
            .context("Invalid redirect URL")?;
        log::debug!("Following redirect to {location}");
        return reqwest::get(location)
            .await
            .context("Failed to download dataset file")?
            .bytes()
            .await
            .context("Failed to read dataset bytes")
            .map(|b| b.to_vec());
    }
    if status.is_success() {
        return response
            .bytes()
            .await
            .context("Failed to read dataset bytes")
            .map(|b| b.to_vec());
    }
    Err(anyhow!(
        "Kaggle download failed: HTTP {status}: {}",
        String::from_utf8_lossy(response.text().await.unwrap_or_default().as_bytes())
    ))
}

fn find_csv_in_zip(zip_data: &[u8]) -> Result<Vec<u8>> {
    let cursor = Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor).context("Failed to open dataset zip")?;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .with_context(|| format!("Failed to read zip entry {i}"))?;
        let name = file.name().to_string();
        if name.ends_with(".csv") {
            let mut buf = Vec::new();
            std::io::copy(&mut file, &mut buf)
                .with_context(|| format!("Failed to extract {name}"))?;
            return Ok(buf);
        }
    }
    Err(anyhow!("No CSV file found in dataset zip"))
}

struct Record {
    text: String,
    row: usize,
}

fn parse_csv(
    csv_data: &[u8],
    column: &str,
    filter_column: Option<&str>,
    filter_value: Option<&str>,
) -> Result<Vec<Record>> {
    let mut reader = csv::Reader::from_reader(csv_data);
    let headers = reader
        .headers()
        .context("Failed to read CSV headers")?
        .clone();

    let col_idx = headers
        .iter()
        .position(|h| h == column)
        .ok_or_else(|| anyhow!("Column '{column}' not found in CSV headers: {headers:?}"))?;

    let filter_idx = filter_column
        .map(|fc| {
            headers.iter().position(|h| h == fc).ok_or_else(|| {
                anyhow!("Filter column '{fc}' not found in CSV headers: {headers:?}")
            })
        })
        .transpose()?;

    let mut records = Vec::new();
    for (row, result) in reader.records().enumerate() {
        let record = result.with_context(|| format!("Failed to read CSV row {row}"))?;
        if let Some(idx) = filter_idx {
            if record.get(idx).unwrap_or("") != filter_value.unwrap_or("") {
                continue;
            }
        }
        let text = record
            .get(col_idx)
            .ok_or_else(|| anyhow!("Row {row}: missing column '{column}'"))?
            .to_string();
        records.push(Record { text, row });
    }
    Ok(records)
}

struct Report {
    total: usize,
    correct: usize,
    wrong: usize,
}

impl Report {
    fn print(&self) {
        let accuracy = if self.total > 0 {
            (self.correct as f64 / self.total as f64) * 100.0
        } else {
            0.0
        };
        println!("Total:    {}", self.total);
        println!("Correct:  {} ({accuracy:.1}%)", self.correct);
        println!("Wrong:    {}", self.wrong);
    }
}

pub async fn run(
    dataset: &str,
    column: &str,
    filter_column: Option<&str>,
    filter_value: Option<&str>,
    expected_spam: bool,
) -> Result<()> {
    let (owner, name) = dataset
        .split_once('/')
        .ok_or_else(|| anyhow!("Dataset must be in 'owner/name' format, got '{dataset}'"))?;

    log::info!("Downloading dataset {owner}/{name}...");
    let zip_data = download_dataset(owner, name).await?;
    log::info!("Extracting CSV from zip...");
    let csv_data = find_csv_in_zip(&zip_data)?;
    let records = parse_csv(&csv_data, column, filter_column, filter_value)?;
    log::info!("Classifying {} records...", records.len());

    let embeddings = Arc::new(Mutex::new(Embeddings::new().await?));
    let classifier = ZeroShotClassification::default(&embeddings).await?;

    let mut report = Report {
        total: 0,
        correct: 0,
        wrong: 0,
    };

    let total = records.len();
    for record in &records {
        let result =
            is_spam_with_custom_classifier(&embeddings, classifier.clone(), &record.text).await?;
        report.total += 1;
        let matched = result.is_spam == expected_spam;
        if matched {
            report.correct += 1;
        } else {
            report.wrong += 1;
            log::error!(
                "row={} is_spam={} expected={} score={:.3} text={}",
                record.row,
                result.is_spam,
                expected_spam,
                result.score.unwrap_or(0.0),
                &record.text.chars().take(80).collect::<String>(),
            );
        }
        if report.total.is_multiple_of(100) {
            eprintln!(
                "{}/{} ({:.0}%)",
                report.total,
                total,
                (report.total as f64 / total as f64) * 100.0
            );
        }
    }

    println!();
    report.print();
    if report.wrong > 0 {
        return Err(anyhow!("{} misclassifications", report.wrong));
    }
    Ok(())
}
