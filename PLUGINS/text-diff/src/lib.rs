//! Text-diff KAMI plugin — compare two texts and return added/removed/unchanged tokens.

#[cfg(target_arch = "wasm32")] mod wasm;
use kami_guest::kami_tool;
use serde::{Deserialize, Serialize};

kami_tool! {
    name: "dev.kami.text-diff",
    version: "0.1.0",
    description: "Compare two texts: returns added/removed/unchanged tokens (lines, words, or chars)",
    handler: handle,
}

/// Input schema for the text-diff plugin.
#[derive(Deserialize)]
struct Input {
    old: String,
    new: String,
    #[serde(default = "default_mode")]
    mode: String,
}

fn default_mode() -> String {
    "lines".to_string()
}

/// A single diff item with its type and value.
#[derive(Serialize)]
struct DiffItem {
    #[serde(rename = "type")]
    kind: String,
    value: String,
}

/// Output schema for the text-diff plugin.
#[derive(Serialize)]
struct Output {
    diff: Vec<DiffItem>,
    added_count: usize,
    removed_count: usize,
    unchanged_count: usize,
    similarity_percent: u8,
}

const MAX_TOKENS: usize = 500;

fn handle(input: &str) -> Result<String, String> {
    let args: Input = kami_guest::parse_input(input)?;
    let (old_tokens, new_tokens): (Vec<String>, Vec<String>) = match args.mode.as_str() {
        "lines" => (
            args.old.lines().map(str::to_string).collect(),
            args.new.lines().map(str::to_string).collect(),
        ),
        "words" => (
            args.old.split_whitespace().map(str::to_string).collect(),
            args.new.split_whitespace().map(str::to_string).collect(),
        ),
        "chars" => (
            args.old.chars().map(|c| c.to_string()).collect(),
            args.new.chars().map(|c| c.to_string()).collect(),
        ),
        other => return Err(format!("unknown mode: {other}")),
    };
    if old_tokens.len() > MAX_TOKENS || new_tokens.len() > MAX_TOKENS {
        return Err(format!(
            "input too large for diff: max {MAX_TOKENS} tokens per side"
        ));
    }
    let diff = compute_diff(&old_tokens, &new_tokens);
    let added = diff.iter().filter(|d| d.kind == "added").count();
    let removed = diff.iter().filter(|d| d.kind == "removed").count();
    let unchanged = diff.iter().filter(|d| d.kind == "unchanged").count();
    let total = old_tokens.len() + new_tokens.len();
    let similarity = if total == 0 {
        100u8
    } else {
        ((2 * unchanged * 100) / total).min(100) as u8
    };
    kami_guest::to_output(&Output {
        diff,
        added_count: added,
        removed_count: removed,
        unchanged_count: unchanged,
        similarity_percent: similarity,
    })
}

/// Compute LCS-based diff between two token sequences.
fn compute_diff(old: &[String], new: &[String]) -> Vec<DiffItem> {
    let m = old.len();
    let n = new.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in (0..m).rev() {
        for j in (0..n).rev() {
            dp[i][j] = if old[i] == new[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let mut result = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < m || j < n {
        if i < m && j < n && old[i] == new[j] {
            result.push(DiffItem { kind: "unchanged".into(), value: old[i].clone() });
            i += 1; j += 1;
        } else if j < n && (i >= m || dp[i + 1][j] >= dp[i][j + 1]) {
            result.push(DiffItem { kind: "added".into(), value: new[j].clone() });
            j += 1;
        } else {
            result.push(DiffItem { kind: "removed".into(), value: old[i].clone() });
            i += 1;
        }
    }
    result
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
