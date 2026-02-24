//! Plugin packaging for `kami build --package`.
//!
//! Creates a distributable `plugin.zip` containing `tool.toml` and the
//! compiled `.wasm` file — exactly what `kami install` expects.

use std::path::{Path, PathBuf};

use kami_types::ToolManifest;

use crate::output;

/// Creates `plugin.zip` containing `tool.toml` and the compiled `.wasm`.
///
/// The archive uses Deflate compression and a flat structure (no
/// subdirectories) matching the `kami install` extraction format.
///
/// # Errors
///
/// Returns an error if any I/O operation fails during packaging.
pub fn package_zip(
    dir: &Path,
    manifest: &ToolManifest,
    wasm_path: &Path,
) -> anyhow::Result<PathBuf> {
    let zip_path = dir.join("plugin.zip");
    let file =
        std::fs::File::create(&zip_path).map_err(|e| anyhow::anyhow!("create zip: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    write_entry(&mut zip, &opts, "tool.toml", &dir.join("tool.toml"))?;
    write_entry(&mut zip, &opts, &manifest.wasm, wasm_path)?;

    zip.finish()
        .map_err(|e| anyhow::anyhow!("finalize zip: {e}"))?;

    output::print_success(&format!("packaged → {}", zip_path.display()));
    Ok(zip_path)
}

/// Adds a single file entry to a zip archive.
fn write_entry(
    zip: &mut zip::ZipWriter<std::fs::File>,
    opts: &zip::write::SimpleFileOptions,
    name: &str,
    path: &Path,
) -> anyhow::Result<()> {
    zip.start_file(name, *opts)
        .map_err(|e| anyhow::anyhow!("zip entry '{name}': {e}"))?;
    let bytes =
        std::fs::read(path).map_err(|e| anyhow::anyhow!("read {}: {e}", path.display()))?;
    std::io::Write::write_all(zip, &bytes)
        .map_err(|e| anyhow::anyhow!("write '{name}' to zip: {e}"))?;
    Ok(())
}
