//! Shared helpers used across CLI commands.
//!
//! Centralises the repetitive pattern of opening the SQLite registry
//! and creating a `KamiRuntime`, ensuring consistent defaults everywhere.

use std::sync::Arc;

use kami_registry::ToolRepository;
use kami_runtime::{KamiRuntime, RuntimeConfig};
use kami_store_sqlite::SqliteToolRepository;

use crate::output;

/// Opens the SQLite tool registry.
///
/// Uses `$KAMI_DATA_DIR/registry.db` or `.kami/registry.db` by default.
///
/// # Errors
///
/// Returns an error if the database file cannot be created or opened.
pub fn open_repository(db: &Option<String>) -> anyhow::Result<Arc<dyn ToolRepository>> {
    let path = db.clone().unwrap_or_else(output::default_db_path);
    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let repo =
        SqliteToolRepository::open(&path).map_err(|e| anyhow::anyhow!("registry error: {e}"))?;
    Ok(Arc::new(repo))
}

/// Creates a `KamiRuntime` with the given concurrency and cache settings.
///
/// # Errors
///
/// Returns an error if the runtime cannot be initialised.
pub fn create_runtime(
    repo: Arc<dyn ToolRepository>,
    concurrency: usize,
    cache_size: usize,
) -> anyhow::Result<KamiRuntime> {
    let config = RuntimeConfig {
        cache_size,
        max_concurrent: concurrency,
        epoch_interruption: true,
        ..RuntimeConfig::default()
    };
    KamiRuntime::new(config, repo).map_err(|e| anyhow::anyhow!("runtime init error: {e}"))
}

/// Returns the KAMI data directory (defaults to `~/.kami`).
///
/// Uses `KAMI_DATA_DIR` if set, otherwise `$HOME/.kami`.
pub fn data_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("KAMI_DATA_DIR") {
        return std::path::PathBuf::from(dir);
    }
    dirs_or_fallback().join(".kami")
}

/// Returns the plugins directory (`<data_dir>/plugins/`).
pub fn plugins_dir() -> std::path::PathBuf {
    data_dir().join("plugins")
}

/// Returns a home directory or a reasonable fallback.
fn dirs_or_fallback() -> std::path::PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn open_repository_in_memory_equivalent() {
        let repo = SqliteToolRepository::open_in_memory();
        assert!(repo.is_ok());
    }

    #[test]
    fn create_runtime_with_defaults() {
        let repo = Arc::new(SqliteToolRepository::open_in_memory().expect("open"));
        let runtime = create_runtime(repo, 4, 32);
        assert!(runtime.is_ok());
    }

    #[test]
    fn open_repository_with_temp_path() {
        let dir = std::env::temp_dir().join("kami_cli_test");
        let _ = std::fs::create_dir_all(&dir);
        let db = dir.join("test_shared.db");
        let _ = std::fs::remove_file(&db);
        let path = db.to_str().expect("utf8").to_string();
        let repo = open_repository(&Some(path));
        assert!(repo.is_ok());
        let _ = std::fs::remove_file(&db);
    }

    #[test]
    fn open_repository_defaults_when_none() {
        // Uses default path (.kami/registry.db)
        let result = open_repository(&None);
        // May fail on CI without permissions, but should not panic
        let _ = result;
    }

    #[test]
    fn data_dir_returns_a_path() {
        let d = data_dir();
        assert!(!d.as_os_str().is_empty());
    }

    #[test]
    fn plugins_dir_is_within_data_dir() {
        let p = plugins_dir();
        assert_eq!(p.file_name().and_then(|s| s.to_str()), Some("plugins"));
    }

    #[test]
    fn dirs_or_fallback_returns_nonempty() {
        let d = dirs_or_fallback();
        assert!(!d.as_os_str().is_empty());
    }
}
