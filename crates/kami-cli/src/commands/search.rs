//! `kami search` command.
//!
//! Searches for tools in the remote registry index.
//! The index is a JSON file hosted on GitHub (or any URL).

use clap::Args;

use crate::output;

/// Default URL for the community tool registry index.
const DEFAULT_INDEX_URL: &str =
    "https://raw.githubusercontent.com/Hypijump31/kami-registry/main/index.json";

/// Search for tools in the remote registry.
#[derive(Debug, Args)]
pub struct SearchArgs {
    /// Search query (matches tool name and description).
    pub query: String,
    /// Custom registry index URL.
    #[arg(long)]
    pub registry: Option<String>,
}

/// A single entry in the remote registry index.
#[derive(Debug, serde::Deserialize)]
struct IndexEntry {
    id: String,
    name: String,
    version: String,
    description: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    download_url: String,
}

/// Executes the search command.
pub async fn execute(args: &SearchArgs) -> anyhow::Result<()> {
    let url = args.registry.as_deref().unwrap_or(DEFAULT_INDEX_URL);

    output::print_info(&format!("Searching registry for '{}'...", args.query));
    let entries = fetch_index(url).await?;
    let query = args.query.to_lowercase();
    let matches: Vec<_> = entries
        .iter()
        .filter(|e| {
            e.name.to_lowercase().contains(&query)
                || e.description.to_lowercase().contains(&query)
                || e.id.to_lowercase().contains(&query)
        })
        .collect();

    if matches.is_empty() {
        output::print_warning(&format!("No tools found matching '{}'", args.query));
        return Ok(());
    }

    output::print_success(&format!("Found {} tool(s):", matches.len()));
    for entry in &matches {
        println!("  {} v{} — {}", entry.id, entry.version, entry.description);
        if !entry.download_url.is_empty() {
            println!("    install: kami install {}", entry.download_url);
        } else if !entry.source.is_empty() {
            println!("    install: kami install {}", entry.source);
        }
    }
    Ok(())
}

/// Fetches and parses the remote registry index.
async fn fetch_index(url: &str) -> anyhow::Result<Vec<IndexEntry>> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| anyhow::anyhow!("failed to fetch registry index: {e}"))?;

    if !resp.status().is_success() {
        anyhow::bail!("registry returned HTTP {}", resp.status());
    }
    let entries: Vec<IndexEntry> = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("invalid registry index JSON: {e}"))?;
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_index_url_is_https() {
        assert!(DEFAULT_INDEX_URL.starts_with("https://"));
    }

    #[test]
    fn index_entry_deserializes() {
        let json = r#"[{
            "id": "dev.test.echo",
            "name": "echo",
            "version": "1.0.0",
            "description": "Echo tool",
            "source": "kami-tools/echo@v1.0.0"
        }]"#;
        let entries: Vec<IndexEntry> = serde_json::from_str(json).expect("deserialize");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "dev.test.echo");
    }

    #[test]
    fn index_entry_without_source() {
        let json = r#"[{
            "id": "test",
            "name": "test",
            "version": "0.1.0",
            "description": "test tool"
        }]"#;
        let entries: Vec<IndexEntry> = serde_json::from_str(json).expect("deserialize");
        assert!(entries[0].source.is_empty());
    }
}
