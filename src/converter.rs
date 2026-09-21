use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn convert_audio_to_pcm(input: &Path) -> Result<PathBuf> {
    let stem = input
        .file_stem()
        .context("Input file has no filename")?
        .to_string_lossy();

    let output = input.with_file_name(format!("{stem}_codecbridge.mov"));
    
    let status = Command::new("ffmpeg")
        .args([
            "-i",
        ])
        .arg(input)
        .args([
            "-map",
            "0:v?",
            "-map",
            "0:a?",
            "-c:v",
            "copy",
            "-c:a",
            "pcm_s24le",
        ])
        .arg(&output)
        .status()
        .context("Failed to run ffmpeg")?;

    if !status.success() {
        anyhow::bail!("ffmpeg conversion failed");
    }

    Ok(output)
}
