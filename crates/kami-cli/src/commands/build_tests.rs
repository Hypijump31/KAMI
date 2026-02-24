//! Tests for `kami build` command.

use super::*;

#[test]
fn build_wasm_fails_without_cargo_toml() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let result = build_wasm(tmp.path(), false);
    assert!(result.is_err());
}

#[test]
fn package_plugin_fails_without_tool_toml() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let result = package_plugin(tmp.path(), false);
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("tool.toml not found"), "got: {msg}");
}
