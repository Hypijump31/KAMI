//! `kami build` — full build pipeline for KAMI tool projects.
//!
//! Validates the manifest, compiles WASM (wasm32-wasip2), verifies the
//! output, computes its SHA-256 hash, and optionally packages a
//! `plugin.zip` ready for distribution.
//!
//! Pipeline steps:
//!
//! 1. **Validate** — parse `tool.toml`, check security config
//! 2. **Compile** — `cargo build --target wasm32-wasip2`
//! 3. **Verify**  — confirm `.wasm` produced, report size
//! 4. **Hash**    — SHA-256 of the generated WASM
//! 5. **Package** — `plugin.zip` (if `--package`)
//! 6. **Report**  — structured summary

use std::path::PathBuf;

use clap::Args;

use super::build_pipeline;
use crate::output;

/// Build a WASM tool from source.
#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Path to the tool project directory (contains Cargo.toml).
    #[arg(default_value = ".")]
    pub tool_dir: String,

    /// Build in release mode (optimised, slower compilation).
    #[arg(long)]
    pub release: bool,

    /// Package the output into plugin.zip (tool.toml + .wasm).
    #[arg(long)]
    pub package: bool,
}

/// Executes the full build pipeline.
///
/// # Errors
///
/// Returns an error if any pipeline step fails (validation, compilation,
/// verification, hashing, or packaging).
pub fn execute(args: &BuildArgs) -> anyhow::Result<()> {
    let dir = PathBuf::from(&args.tool_dir)
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("invalid tool directory '{}': {e}", args.tool_dir))?;

    // Step 1 — Validate manifest + security config.
    output::print_info("validating tool.toml…");
    let manifest = build_pipeline::validate_manifest(&dir)?;
    output::print_success(&format!(
        "{} v{} — {}",
        manifest.id, manifest.version, manifest.description
    ));

    // Step 2 — Compile WASM.
    let profile = if args.release { "release" } else { "debug" };
    output::print_info(&format!("compiling ({profile})…"));
    build_pipeline::compile_wasm(&dir, args.release)?;

    // Step 3 — Verify produced .wasm.
    let wasm_path = build_pipeline::locate_wasm(&dir, &manifest, args.release)?;
    let wasm_size = build_pipeline::file_size(&wasm_path)?;

    // Step 4 — Compute SHA-256.
    let sha256 = build_pipeline::compute_wasm_hash(&wasm_path)?;

    // Step 5 — Package (optional).
    let zip_path = if args.package {
        Some(build_pipeline::package_zip(&dir, &manifest, &wasm_path)?)
    } else {
        None
    };

    // Step 6 — Report.
    let report = build_pipeline::BuildReport {
        tool_id: manifest.id.to_string(),
        tool_name: manifest.name.clone(),
        version: manifest.version.to_string(),
        wasm_file: manifest.wasm.clone(),
        wasm_size,
        wasm_sha256: sha256,
        profile: profile.to_string(),
        zip_path,
    };
    print_report(&report);

    Ok(())
}

/// Prints a human-readable build summary.
fn print_report(r: &build_pipeline::BuildReport) {
    println!();
    println!("  Tool:    {} ({})", r.tool_id, r.tool_name);
    println!("  Version: {}", r.version);
    println!("  WASM:    {} ({} bytes)", r.wasm_file, r.wasm_size);
    println!("  SHA-256: {}", r.wasm_sha256);
    println!("  Profile: {}", r.profile);
    if let Some(zip) = &r.zip_path {
        println!("  Package: {}", zip.display());
    }
    println!();
    output::print_success("build complete");
}

#[cfg(test)]
#[path = "build_tests.rs"]
mod tests;
