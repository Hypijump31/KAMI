//! Regex-extract KAMI plugin — extract capture groups from text using regex.

#[cfg(target_arch = "wasm32")] mod wasm;
use kami_guest::kami_tool;
use regex::Regex;
use serde::{Deserialize, Serialize};

kami_tool! {
    name: "dev.kami.regex-extract",
    version: "0.1.0",
    description: "Extract regex capture groups from text",
    handler: handle,
}

/// Input schema for the regex-extract plugin.
#[derive(Deserialize)]
struct Input {
    pattern: String,
    text: String,
    #[serde(default = "default_all")]
    all: bool,
}

fn default_all() -> bool {
    true
}

/// A single regex match with its position and captured groups.
#[derive(Serialize)]
struct Match {
    full: String,
    groups: Vec<Option<String>>,
    start: usize,
    end: usize,
}

/// Output schema for the regex-extract plugin.
#[derive(Serialize)]
struct Output {
    matches: Vec<Match>,
    count: usize,
}

fn handle(input: &str) -> Result<String, String> {
    let args: Input = kami_guest::parse_input(input)?;
    let re = Regex::new(&args.pattern)
        .map_err(|e| format!("invalid pattern: {e}"))?;
    let matches = if args.all {
        find_all(&re, &args.text)
    } else {
        find_first(&re, &args.text)
    };
    let count = matches.len();
    kami_guest::to_output(&Output { matches, count })
}

/// Find all non-overlapping matches with their capture groups.
fn find_all(re: &Regex, text: &str) -> Vec<Match> {
    re.captures_iter(text)
        .map(|cap| build_match(&cap, text))
        .collect()
}

/// Find only the first match.
fn find_first(re: &Regex, text: &str) -> Vec<Match> {
    re.captures(text)
        .map(|cap| vec![build_match(&cap, text)])
        .unwrap_or_default()
}

fn build_match(cap: &regex::Captures<'_>, _text: &str) -> Match {
    let full_match = cap.get(0).map_or("", |m| m.as_str());
    let (start, end) = cap
        .get(0)
        .map(|m| (m.start(), m.end()))
        .unwrap_or((0, 0));
    let groups = (1..cap.len())
        .map(|i| cap.get(i).map(|m| m.as_str().to_string()))
        .collect();
    Match { full: full_match.to_string(), groups, start, end }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_all_emails() {
        let result = handle(
            r#"{"pattern":"\\w+@\\w+\\.\\w+","text":"a@b.com and c@d.org","all":true}"#,
        )
        .expect("handle");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["count"], 2);
    }

    #[test]
    fn find_first_only() {
        let result = handle(
            r#"{"pattern":"\\d+","text":"1 and 2 and 3","all":false}"#,
        )
        .expect("handle");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["count"], 1);
        assert_eq!(v["matches"][0]["full"], "1");
    }

    #[test]
    fn no_match_returns_empty() {
        let result = handle(r#"{"pattern":"xyz","text":"hello world"}"#).expect("handle");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["count"], 0);
    }

    #[test]
    fn capture_groups_extracted() {
        let result = handle(
            r#"{"pattern":"(\\w+)@(\\w+)","text":"alice@example"}"#,
        )
        .expect("handle");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        let groups = &v["matches"][0]["groups"];
        assert_eq!(groups[0], "alice");
        assert_eq!(groups[1], "example");
    }

    #[test]
    fn invalid_pattern_returns_error() {
        let result = handle(r#"{"pattern":"[invalid","text":"test"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid pattern"));
    }

    #[test]
    fn empty_text_returns_empty_result() {
        let result = handle(r#"{"pattern":"\\d+","text":""}"#).expect("handle");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["count"], 0);
    }
}
