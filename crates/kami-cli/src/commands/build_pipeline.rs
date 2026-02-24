//! Build pipeline steps for `kami build`.
//!
//! Each public function represents an atomic, independently testable step.
//! The orchestrator in [`super::build`] calls them in sequence and collects
//! results into a [`BuildReport`].
//!
//! Packaging logic lives in [`super::build_package`] to respect the
//! 150-line module limit.

use std::path::{Path, PathBuf};

use kami_config::parse_tool_manifest_file;
use kami_runtime::compute_file_hash;
use kami_sandbox::validate_security_config;
use kami_types::ToolManifest;

use crate::output;

pub use super::build_package::package_zip;

/// Summary of a completed build, suitable for structured output.
#[derive(Debug)]
pub struct BuildReport {
    /// Fully qualified tool identifier (e.g. `dev.kami.hash-compute`).
    pub tool_id: String,
    /// Human-readable tool name.
    pub tool_name: String,
    /// Semantic version string.
    pub version: String,
    /// WASM filename as declared in `tool.toml`.
    pub wasm_file: String,
    /// Size of the compiled WASM in bytes.
    pub wasm_size: u64,
    /// SHA-256 hex digest of the compiled WASM.
    pub wasm_sha256: String,
    /// Build profile (`debug` or `release`).
    pub profile: String,
    /// Path to the generated `plugin.zip`, if packaging was requested.
    pub zip_path: Option<PathBuf>,
}

// ── Step 1: Validate ────────────────────────────────────────────────

/// Parses `tool.toml` and validates the security configuration.
///
/// # Errors
///
/// Returns an error if `tool.toml` is missing, malformed, or contains
/// an invalid security configuration.
pub fn validate_manifest(dir: &Path) -> anyhow::Result<ToolManifest> {
    let toml_path = dir.join("tool.toml");
    if !toml_path.exists() {
        anyhow::bail!("tool.toml not found in {}", dir.display());
    }

    let manifest = parse_tool_manifest_file(&toml_path)
        .map_err(|e| anyhow::anyhow!("manifest error: {e}"))?;

    validate_security_config(&manifest.security)
        .map_err(|e| anyhow::anyhow!("security config error: {e}"))?;

    output::print_info(&format!(
        "security: net={} hosts, fs={:?}, mem={}MB, timeout={}ms",
        manifest.security.net_allow_list.len(),
        manifest.security.fs_access,
        manifest.security.limits.max_memory_mb,
        manifest.security.limits.max_execution_ms,
    ));

    for arg in &manifest.arguments {
        let req = if arg.required { "required" } else { "optional" };
        output::print_info(&format!("  arg: {} ({}, {})", arg.name, arg.arg_type, req));
    }

    Ok(manifest)
}

// ── Step 2: Compile ─────────────────────────────────────────────────

/// Spawns `cargo build --target wasm32-wasip2` in `dir`.
///
/// # Errors
///
/// Returns an error if `cargo` is not found or the build fails.
pub fn compile_wasm(dir: &Path, release: bool) -> anyhow::Result<()> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build")
        .arg("--target")
        .arg("wasm32-wasip2")
        .current_dir(dir);
    if release {
        cmd.arg("--release");
    }
    let status = cmd
        .status()
        .map_err(|e| anyhow::anyhow!("failed to spawn cargo: {e}"))?;
    if !status.success() {
        anyhow::bail!("cargo build exited with {status}");
    }
    Ok(())
}

// ── Step 3: Verify ──────────────────────────────────────────────────

/// Returns the path to the compiled `.wasm` file.
///
/// # Errors
///
/// Returns an error if the expected WASM output does not exist.
pub fn locate_wasm(
    dir: &Path,
    manifest: &ToolManifest,
    release: bool,
) -> anyhow::Result<PathBuf> {
    let profile = if release { "release" } else { "debug" };
    let wasm_path = dir
        .join("target")
        .join("wasm32-wasip2")
        .join(profile)
        .join(&manifest.wasm);

    if !wasm_path.exists() {
        anyhow::bail!(
            "WASM output not found: {} (expected at {})",
            manifest.wasm,
            wasm_path.display()
        );
    }

    Ok(wasm_path)
}

/// Returns the size in bytes of a file.
///
/// # Errors
///
/// Returns an error if the file metadata cannot be read.
pub fn file_size(path: &Path) -> anyhow::Result<u64> {
    let meta = std::fs::metadata(path)
        .map_err(|e| anyhow::anyhow!("cannot stat {}: {e}", path.display()))?;
    Ok(meta.len())
}

// ── Step 4: Hash ────────────────────────────────────────────────────

/// Computes the SHA-256 hex digest of a WASM file.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub fn compute_wasm_hash(wasm_path: &Path) -> anyhow::Result<String> {
    let hash = compute_file_hash(wasm_path)
        .map_err(|e| anyhow::anyhow!("hash error for {}: {e}", wasm_path.display()))?;
    output::print_success(&format!("SHA-256: {hash}"));
    Ok(hash)
}
