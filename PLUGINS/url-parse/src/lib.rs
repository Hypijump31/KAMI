//! Url-parse KAMI plugin — parse URL components and extract query parameters.

#[cfg(target_arch = "wasm32")] mod wasm;
use kami_guest::kami_tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

kami_tool! {
    name: "dev.kami.url-parse",
    version: "0.1.0",
    description: "Parse URL components: scheme, host, port, path, query params, fragment",
    handler: handle,
}

/// Input schema for the url-parse plugin.
#[derive(Deserialize)]
struct Input {
    url: String,
    #[serde(default = "default_action")]
    action: String,
}

fn default_action() -> String {
    "parse".to_string()
}

/// Full URL breakdown output.
#[derive(Serialize)]
struct ParsedUrl {
    valid: bool,
    scheme: String,
    host: Option<String>,
    port: Option<u16>,
    path: String,
    query: Option<String>,
    query_params: HashMap<String, String>,
    fragment: Option<String>,
    origin: String,
}

fn handle(input: &str) -> Result<String, String> {
    let args: Input = kami_guest::parse_input(input)?;
    match args.action.as_str() {
        "parse" => parse_url(&args.url),
        "validate" => validate_url(&args.url),
        "query_params" => extract_query_params(&args.url),
        other => Err(format!("unknown action: {other}")),
    }
}

fn parse_url(raw: &str) -> Result<String, String> {
    let url = Url::parse(raw).map_err(|e| format!("invalid URL: {e}"))?;
    let query_params = parse_query(url.query().unwrap_or(""));
    let origin = format!(
        "{}://{}{}",
        url.scheme(),
        url.host_str().unwrap_or(""),
        url.port().map_or(String::new(), |p| format!(":{p}"))
    );
    kami_guest::to_output(&ParsedUrl {
        valid: true,
        scheme: url.scheme().to_string(),
        host: url.host_str().map(str::to_string),
        port: url.port(),
        path: url.path().to_string(),
        query: url.query().map(str::to_string),
        query_params,
        fragment: url.fragment().map(str::to_string),
        origin,
    })
}

fn validate_url(raw: &str) -> Result<String, String> {
    let err = Url::parse(raw).err().map(|e| e.to_string());
    kami_guest::to_output(&serde_json::json!({"valid": err.is_none(), "error": err}))
}

fn extract_query_params(raw: &str) -> Result<String, String> {
    let url = Url::parse(raw).map_err(|e| format!("invalid URL: {e}"))?;
    let params = parse_query(url.query().unwrap_or(""));
    kami_guest::to_output(&params)
}

fn parse_query(query: &str) -> HashMap<String, String> {
    query.split('&').filter(|s| !s.is_empty()).filter_map(|pair| {
        let mut p = pair.splitn(2, '=');
        Some((p.next()?.to_string(), p.next().unwrap_or("").to_string()))
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_url() {
        let result = parse_url("https://api.example.com:8080/v1?page=2#top").expect("parse");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["scheme"], "https");
        assert_eq!(v["host"], "api.example.com");
        assert_eq!(v["port"], 8080);
        assert_eq!(v["path"], "/v1");
        assert_eq!(v["fragment"], "top");
    }

    #[test]
    fn parse_url_without_port() {
        let result = parse_url("https://example.com/").expect("parse");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert!(v["port"].is_null());
    }

    #[test]
    fn query_params_extraction() {
        let result = extract_query_params("https://x.com?a=1&b=2").expect("q");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["a"], "1");
        assert_eq!(v["b"], "2");
    }

    #[test]
    fn validate_valid_url() {
        let result = validate_url("https://example.com").expect("v");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["valid"], true);
    }

    #[test]
    fn validate_invalid_url() {
        let result = validate_url("not a url").expect("v");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["valid"], false);
        assert!(!v["error"].is_null());
    }

    #[test]
    fn invalid_url_parse_returns_error() {
        let result = parse_url(":::invalid");
        assert!(result.is_err());
    }

    #[test]
    fn unknown_action_returns_error() {
        let result = handle(r#"{"url":"https://x.com","action":"nope"}"#);
        assert!(result.is_err());
    }
}
