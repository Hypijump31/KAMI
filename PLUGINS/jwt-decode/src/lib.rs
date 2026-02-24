//! Jwt-decode KAMI plugin — decode JWT tokens and inspect claims (read-only).

#[cfg(target_arch = "wasm32")] mod wasm;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use kami_guest::kami_tool;
use serde::{Deserialize, Serialize};
use serde_json::Value;

kami_tool! {
    name: "dev.kami.jwt-decode",
    version: "0.1.0",
    description: "Decode a JWT token and inspect its header and payload (no signature verification)",
    handler: handle,
}

/// Input schema for the jwt-decode plugin.
#[derive(Deserialize)]
struct Input {
    token: String,
}

/// Output schema for the jwt-decode plugin.
#[derive(Serialize)]
struct Output {
    header: Value,
    payload: Value,
    signature: String,
    /// Always null in V1 — WASM has no clock access without WASI clocks configured.
    expired: Option<bool>,
    algorithm: Option<String>,
}

fn handle(input: &str) -> Result<String, String> {
    let args: Input = kami_guest::parse_input(input)?;
    let output = decode_jwt(&args.token)?;
    kami_guest::to_output(&output)
}

/// Decode the three parts of a JWT token.
fn decode_jwt(token: &str) -> Result<Output, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(format!(
            "invalid JWT format: expected 3 parts, got {}",
            parts.len()
        ));
    }
    let header = decode_part(parts[0]).map_err(|e| format!("invalid JWT header: {e}"))?;
    let payload = decode_part(parts[1]).map_err(|e| format!("invalid JWT payload: {e}"))?;
    let algorithm = header
        .get("alg")
        .and_then(Value::as_str)
        .map(str::to_string);
    Ok(Output {
        header,
        payload,
        signature: parts[2].to_string(),
        expired: None,
        algorithm,
    })
}

/// Decode a Base64url-encoded JWT part into a JSON value.
fn decode_part(part: &str) -> Result<Value, String> {
    let padded = pad_base64url(part);
    let bytes = URL_SAFE_NO_PAD
        .decode(part)
        .or_else(|_| {
            let engine = base64::engine::general_purpose::URL_SAFE;
            engine.decode(&padded)
        })
        .map_err(|e| format!("invalid base64: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("invalid JSON: {e}"))
}

/// Add padding to a Base64url string if needed.
fn pad_base64url(s: &str) -> String {
    let pad = (4 - s.len() % 4) % 4;
    format!("{}{}", s, "=".repeat(pad))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TOKEN: &str = concat!(
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9",
        ".",
        "eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ",
        ".",
        "SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"
    );

    #[test]
    fn decode_valid_jwt() {
        let output = decode_jwt(SAMPLE_TOKEN).expect("decode");
        assert_eq!(output.payload["name"], "John Doe");
        assert_eq!(output.payload["sub"], "1234567890");
    }

    #[test]
    fn extract_algorithm() {
        let output = decode_jwt(SAMPLE_TOKEN).expect("decode");
        assert_eq!(output.algorithm.as_deref(), Some("HS256"));
    }

    #[test]
    fn expired_is_null_in_v1() {
        let output = decode_jwt(SAMPLE_TOKEN).expect("decode");
        assert!(output.expired.is_none());
    }

    #[test]
    fn invalid_format_two_parts() {
        let result = decode_jwt("aaa.bbb");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected 3 parts"));
    }

    #[test]
    fn invalid_base64_returns_error() {
        let result = decode_jwt("!!.!!!!.!!");
        assert!(result.is_err());
    }

    #[test]
    fn signature_preserved() {
        let output = decode_jwt(SAMPLE_TOKEN).expect("decode");
        assert_eq!(output.signature, "SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c");
    }
}
