//! Base64-codec KAMI plugin — encode and decode Base64 (standard and URL-safe).

#[cfg(target_arch = "wasm32")] mod wasm;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use kami_guest::kami_tool;
use serde::{Deserialize, Serialize};

kami_tool! {
    name: "dev.kami.base64-codec",
    version: "0.1.0",
    description: "Encode or decode Base64 (standard and URL-safe variants)",
    handler: handle,
}

/// Input schema for the base64-codec plugin.
#[derive(Deserialize)]
struct Input {
    action: String,
    data: String,
    #[serde(default)]
    url_safe: bool,
}

/// Output schema for the base64-codec plugin.
#[derive(Serialize)]
struct Output {
    result: String,
    action: String,
    url_safe: bool,
}

fn handle(input: &str) -> Result<String, String> {
    let args: Input = kami_guest::parse_input(input)?;
    let result = match args.action.as_str() {
        "encode" => encode_base64(&args.data, args.url_safe),
        "decode" => decode_base64(&args.data, args.url_safe)?,
        other => return Err(format!("unknown action: {other}")),
    };
    kami_guest::to_output(&Output {
        result,
        action: args.action,
        url_safe: args.url_safe,
    })
}

/// Encode a string to Base64.
fn encode_base64(data: &str, url_safe: bool) -> String {
    if url_safe {
        URL_SAFE_NO_PAD.encode(data.as_bytes())
    } else {
        STANDARD.encode(data.as_bytes())
    }
}

/// Decode a Base64 string, trying standard then URL-safe on failure.
fn decode_base64(data: &str, url_safe: bool) -> Result<String, String> {
    let bytes = if url_safe {
        URL_SAFE_NO_PAD
            .decode(data)
            .map_err(|e| format!("invalid base64: {e}"))?
    } else {
        STANDARD
            .decode(data)
            .or_else(|_| URL_SAFE_NO_PAD.decode(data))
            .map_err(|e| format!("invalid base64: {e}"))?
    };
    String::from_utf8(bytes).map_err(|e| format!("decoded bytes are not valid UTF-8: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_standard() {
        assert_eq!(encode_base64("hello world", false), "aGVsbG8gd29ybGQ=");
    }

    #[test]
    fn encode_url_safe() {
        let encoded = encode_base64("hello world", true);
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
        assert!(!encoded.contains('='));
    }

    #[test]
    fn decode_standard() {
        let decoded = decode_base64("aGVsbG8gd29ybGQ=", false).expect("decode");
        assert_eq!(decoded, "hello world");
    }

    #[test]
    fn roundtrip_encode_then_decode() {
        let original = "Hello, KAMI! 🦀";
        let encoded = encode_base64(original, false);
        let decoded = decode_base64(&encoded, false).expect("decode");
        assert_eq!(decoded, original);
    }

    #[test]
    fn unknown_action_returns_error() {
        let result = handle(r#"{"action":"xor","data":"test"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown action"));
    }

    #[test]
    fn invalid_base64_decode_returns_error() {
        let result = decode_base64("not!!valid$$base64", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid base64"));
    }

    #[test]
    fn empty_string_roundtrip() {
        let encoded = encode_base64("", false);
        let decoded = decode_base64(&encoded, false).expect("decode");
        assert_eq!(decoded, "");
    }
}
