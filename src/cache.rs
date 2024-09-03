use airnope::{embeddings, summary};
use anyhow::{anyhow, Result};
use dirs::cache_dir;
use std::{env, path::PathBuf};
use tokio::{fs::remove_dir_all, try_join};
use walkdir::WalkDir;

fn format_size(size: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < units.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.2} {}", size, units[unit])
}

pub async fn clean_rust_bert_cache(dry_run: bool) -> Result<()> {
    let dir = match env::var("RUSTBERT_CACHE") {
        Ok(value) => PathBuf::from(value),
        Err(_) => {
            let mut cache = cache_dir().ok_or(anyhow!("Could not find the cache directory"))?;
            cache.push(".rustbert");
            cache
        }
    };
    let mut label = if dry_run { "Checking" } else { "Deleting" };
    log::info!("{} {}", label, dir.as_os_str().to_string_lossy());
    let size = WalkDir::new(&dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.metadata().map(|m| m.is_file()).unwrap_or(false))
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum::<u64>();
    if dry_run {
        label = "Total size";
    } else {
        remove_dir_all(&dir).await?;
        label = "Cleaned up";
    }
    log::info!("{} {}", label, format_size(size));
    Ok(())
}

pub async fn download_all() -> Result<()> {
    try_join!(summary::download(), embeddings::download())?;
    Ok(())
}
