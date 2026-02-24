//! `kami build` command.
//!
//! Compiles a tool project to WASM (wasm32-wasip2 target) with a single
//! invocation.  Optionally packages the result into a `plugin.zip` ready
//! for distribution.

use std::path::{Path, PathBuf};

use clap::Args;

/// Build a WASM tool from source.
#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Path to the tool project directory (contains Cargo.toml).
    #[arg(default_value = ".")]
    pub tool_dir: String,
    /// Build in release mode.
    #[arg(long)]
    pub release: bool,
    /// Package the output into plugin.zip (tool.toml + .wasm).
    #[arg(long)]
    pub package: bool,
}

/// Executes the build command.
///
/// # Errors
///
/// Returns an error if the cargo build fails or packaging encounters I/O
/// issues.
pub fn execute(args: &BuildArgs) -> anyhow::Result<()> {
    let dir = PathBuf::from(&args.tool_dir)
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("invalid tool directory '{}': {e}", args.tool_dir))?;

    println!("[BUILD] compiling {}", dir.display());
    build_wasm(&dir, args.release)?;

    let profile = if args.release { "release" } else { "debug" };
    println!("[OK] build succeeded ({})", profile);

    if args.package {
        package_plugin(&dir, args.release)?;
    }

    Ok(())
}

/// Runs `cargo build --target wasm32-wasip2` in the given directory.
fn build_wasm(dir: &Path, release: bool) -> anyhow::Result<()> {
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
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("cargo build exited with {status}"))
    }
}

/// Creates a `plugin.zip` containing tool.toml and the compiled .wasm.
fn package_plugin(dir: &Path, release: bool) -> anyhow::Result<()> {
    let tool_toml = dir.join("tool.toml");
    if !tool_toml.exists() {
        anyhow::bail!("tool.toml not found in {}", dir.display());
    }

    let manifest = kami_config::parse_tool_manifest_file(&tool_toml)
        .map_err(|e| anyhow::anyhow!("failed to parse tool.toml: {e}"))?;

    let profile = if release { "release" } else { "debug" };
    let wasm_path = dir
        .join("target")
        .join("wasm32-wasip2")
        .join(profile)
        .join(&manifest.wasm);

    if !wasm_path.exists() {
        anyhow::bail!("WASM file not found: {}", wasm_path.display());
    }

    let zip_path = dir.join("plugin.zip");
    let file =
        std::fs::File::create(&zip_path).map_err(|e| anyhow::anyhow!("create zip: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Add tool.toml
    zip.start_file("tool.toml", options)
        .map_err(|e| anyhow::anyhow!("zip tool.toml: {e}"))?;
    let toml_bytes = std::fs::read(&tool_toml)
        .map_err(|e| anyhow::anyhow!("read tool.toml: {e}"))?;
    std::io::Write::write_all(&mut zip, &toml_bytes)
        .map_err(|e| anyhow::anyhow!("write tool.toml to zip: {e}"))?;

    // Add .wasm
    let wasm_name = &manifest.wasm;
    zip.start_file(wasm_name, options)
        .map_err(|e| anyhow::anyhow!("zip {wasm_name}: {e}"))?;
    let wasm_bytes = std::fs::read(&wasm_path)
        .map_err(|e| anyhow::anyhow!("read {wasm_name}: {e}"))?;
    std::io::Write::write_all(&mut zip, &wasm_bytes)
        .map_err(|e| anyhow::anyhow!("write {wasm_name} to zip: {e}"))?;

    zip.finish()
        .map_err(|e| anyhow::anyhow!("finalize zip: {e}"))?;

    println!("[OK] packaged → {}", zip_path.display());
    Ok(())
}

#[cfg(test)]
#[path = "build_tests.rs"]
mod tests;
