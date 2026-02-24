//! Hash-compute KAMI plugin — compute SHA-256 or SHA-512 of any text.

#[cfg(target_arch = "wasm32")] mod wasm;
use kami_guest::kami_tool;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};

kami_tool! {
    name: "dev.kami.hash-compute",
    version: "0.1.0",
    description: "Compute SHA-256 or SHA-512 hash of any text",
    handler: handle,
}

/// Input schema for the hash-compute plugin.
#[derive(Deserialize)]
struct Input {
    text: String,
    #[serde(default = "default_algorithm")]
    algorithm: String,
}

/// Output schema for the hash-compute plugin.
#[derive(Serialize)]
struct Output {
    hash: String,
    algorithm: String,
    input_length: usize,
}

fn default_algorithm() -> String {
    "sha256".to_string()
}

fn handle(input: &str) -> Result<String, String> {
    let args: Input = kami_guest::parse_input(input)?;
    let hash = match args.algorithm.as_str() {
        "sha256" => compute_sha256(&args.text),
        "sha512" => compute_sha512(&args.text),
        other => return Err(format!("unknown algorithm: {other}")),
    };
    kami_guest::to_output(&Output {
        input_length: args.text.len(),
        hash,
        algorithm: args.algorithm,
    })
}

/// Compute the hex-encoded SHA-256 hash of the given text.
fn compute_sha256(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

/// Compute the hex-encoded SHA-512 hash of the given text.
fn compute_sha512(text: &str) -> String {
    let mut hasher = Sha512::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_of_known_string() {
        let hash = compute_sha256("hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn sha512_of_known_string() {
        let hash = compute_sha512("hello");
        assert!(hash.starts_with("9b71d224bd62f378"));
        assert_eq!(hash.len(), 128);
    }

    #[test]
    fn default_algorithm_is_sha256() {
        let result = handle(r#"{"text":"hello"}"#).expect("handle");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["algorithm"], "sha256");
    }

    #[test]
    fn sha512_via_algorithm_field() {
        let result = handle(r#"{"text":"hello","algorithm":"sha512"}"#).expect("handle");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["algorithm"], "sha512");
        assert_eq!(v["hash"].as_str().expect("str").len(), 128);
    }

    #[test]
    fn unknown_algorithm_returns_error() {
        let result = handle(r#"{"text":"hello","algorithm":"md5"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown algorithm"));
    }

    #[test]
    fn empty_string_hashes_correctly() {
        let hash = compute_sha256("");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn input_length_is_reported() {
        let result = handle(r#"{"text":"hello"}"#).expect("handle");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["input_length"], 5);
    }
}
