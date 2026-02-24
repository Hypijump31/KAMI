//! Tests for `kami build` pipeline steps.

use super::build_pipeline;

#[test]
fn validate_manifest_fails_without_tool_toml() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let err = build_pipeline::validate_manifest(tmp.path());
    assert!(err.is_err());
    let msg = format!("{}", err.unwrap_err());
    assert!(msg.contains("tool.toml not found"), "got: {msg}");
}

#[test]
fn validate_manifest_rejects_invalid_security() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    // max_memory_mb = 0 triggers security validation failure
    let toml = r#"
[tool]
id = "dev.test.bad-sec"
name = "bad-sec"
version = "1.0.0"
wasm = "bad.wasm"

[mcp]
description = "bad security"

[security]
fs_access = "none"
max_memory_mb = 0
max_execution_ms = 1000
"#;
    std::fs::write(tmp.path().join("tool.toml"), toml).expect("write");
    let err = build_pipeline::validate_manifest(tmp.path());
    assert!(err.is_err());
    let msg = format!("{}", err.unwrap_err());
    assert!(msg.contains("security"), "got: {msg}");
}

#[test]
fn validate_manifest_accepts_valid_tool() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let toml = r#"
[tool]
id = "dev.test.ok"
name = "ok"
version = "1.0.0"
wasm = "ok.wasm"

[mcp]
description = "valid"

[security]
fs_access = "none"
max_memory_mb = 16
max_execution_ms = 1000
"#;
    std::fs::write(tmp.path().join("tool.toml"), toml).expect("write");
    let manifest = build_pipeline::validate_manifest(tmp.path());
    assert!(manifest.is_ok());
    let m = manifest.unwrap();
    assert_eq!(m.name, "ok");
}

#[test]
fn compile_wasm_fails_without_cargo_toml() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let err = build_pipeline::compile_wasm(tmp.path(), false);
    assert!(err.is_err());
}

#[test]
fn locate_wasm_fails_when_missing() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let toml = r#"
[tool]
id = "dev.test.loc"
name = "loc"
version = "1.0.0"
wasm = "loc.wasm"

[mcp]
description = "locate test"

[security]
fs_access = "none"
max_memory_mb = 16
max_execution_ms = 1000
"#;
    std::fs::write(tmp.path().join("tool.toml"), toml).expect("write");
    let manifest =
        kami_config::parse_tool_manifest_file(&tmp.path().join("tool.toml")).expect("parse");
    let err = build_pipeline::locate_wasm(tmp.path(), &manifest, false);
    assert!(err.is_err());
    let msg = format!("{}", err.unwrap_err());
    assert!(msg.contains("WASM output not found"), "got: {msg}");
}

#[test]
fn file_size_returns_correct_value() {
    let tmp = tempfile::NamedTempFile::new().expect("tmpfile");
    std::fs::write(tmp.path(), b"hello").expect("write");
    let size = build_pipeline::file_size(tmp.path()).expect("size");
    assert_eq!(size, 5);
}

#[test]
fn compute_wasm_hash_is_deterministic() {
    let tmp = tempfile::NamedTempFile::new().expect("tmpfile");
    std::fs::write(tmp.path(), b"wasm content").expect("write");
    let h1 = build_pipeline::compute_wasm_hash(tmp.path()).expect("h1");
    let h2 = build_pipeline::compute_wasm_hash(tmp.path()).expect("h2");
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64, "SHA-256 hex = 64 chars");
}

#[test]
fn package_zip_fails_without_tool_toml() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let fake_wasm = tmp.path().join("fake.wasm");
    std::fs::write(&fake_wasm, b"fake").expect("write");

    let toml = r#"
[tool]
id = "dev.test.pkg"
name = "pkg"
version = "1.0.0"
wasm = "fake.wasm"

[mcp]
description = "pkg test"

[security]
fs_access = "none"
max_memory_mb = 16
max_execution_ms = 1000
"#;
    // Parse manifest from string (tool.toml must also exist on disk for zip)
    let manifest = kami_config::parse_tool_manifest(toml).expect("parse");

    // No tool.toml on disk → write_entry will fail
    let err = build_pipeline::package_zip(tmp.path(), &manifest, &fake_wasm);
    assert!(err.is_err());
}

#[test]
fn package_zip_creates_valid_archive() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let toml = r#"
[tool]
id = "dev.test.zip"
name = "zip-test"
version = "1.0.0"
wasm = "test.wasm"

[mcp]
description = "zip test"

[security]
fs_access = "none"
max_memory_mb = 16
max_execution_ms = 1000
"#;
    std::fs::write(tmp.path().join("tool.toml"), toml).expect("write toml");
    let fake_wasm = tmp.path().join("test.wasm");
    std::fs::write(&fake_wasm, b"fake wasm data").expect("write wasm");

    let manifest =
        kami_config::parse_tool_manifest_file(&tmp.path().join("tool.toml")).expect("parse");

    let zip_path =
        build_pipeline::package_zip(tmp.path(), &manifest, &fake_wasm).expect("package");
    assert!(zip_path.exists());

    // Verify zip contents
    let file = std::fs::File::open(&zip_path).expect("open zip");
    let mut archive = zip::ZipArchive::new(file).expect("read zip");
    assert_eq!(archive.len(), 2);

    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).expect("entry").name().to_string())
        .collect();
    assert!(names.contains(&"tool.toml".to_string()));
    assert!(names.contains(&"test.wasm".to_string()));
}
