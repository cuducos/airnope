use airnope::embeddings;
use anyhow::Result;
use hf_hub::Cache;
use tokio::fs::remove_dir_all;
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

pub async fn clean_model_cache(dry_run: bool) -> Result<()> {
    let cache = Cache::default();
    let dir = cache.path();

    let mut label = if dry_run { "Checking" } else { "Deleting" };
    log::info!("{} {}", label, dir.display());

    let size = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().map(|m| m.is_file()).unwrap_or(false))
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum::<u64>();

    if dry_run {
        label = "Total size";
    } else {
        if dir.exists() {
            remove_dir_all(&dir).await?;
        }
        label = "Cleaned up";
    }

    log::info!("{} {}", label, format_size(size));
    Ok(())
}

pub async fn download_all() -> Result<()> {
    embeddings::download().await
}
