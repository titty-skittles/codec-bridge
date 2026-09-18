use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "codecbridge")]
#[command(about = "Media compatibility helper")]
struct Args {
    /// Media file to inspect
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ])
        .arg(&args.input)
        .output()
        .context("Failed to run ffprobe")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffprobe failed:\n{stderr}");
    }

    let stdout = 
        String::from_utf8(output.stdout)
            .context("ffprobe output was not valid UTF-8")?;

    println!("{stdout}");

    Ok(())
}
