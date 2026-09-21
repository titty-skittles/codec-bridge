use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::planner::{AudioAction, ConversionPlan, VideoAction};

pub fn convert_file(input: &Path, plan: &ConversionPlan) -> Result<PathBuf> {
    let stem = input
        .file_stem()
        .context("Input file has no filename")?
        .to_string_lossy();

    let output = input.with_file_name(format!("{stem}_codecbridge.mov"));

    let mut command = Command::new("ffmpeg");
    
    command
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
        ])
        .arg(input)
        .args([
            "-map",
            "0:v?",
            "-map",
            "0:a?",
        ]);

    match plan.video {
        VideoAction::Copy => {
            command.args(["-c:v", "copy"]);
        }

        VideoAction::TranscodeDnxhr => {
            command.args([
                "-c:v",
                "dnxhd",
                "-profile:v",
                "dnxhr_hq",
            ]);
        }
    }

    match plan.audio {
        AudioAction::Copy => {
            command.args(["-c:a", "copy"]);
        }

        AudioAction::ConvertToPcm => {
            command.args(["-c:a", "pcm_s24le"]);
        }
    }

    let status = command
        .arg(&output)
        .status()
        .context("Failed to run ffmpeg")?;

    if !status.success() {
        anyhow::bail!("ffmpeg conversion failed");
    }

    Ok(output)
}

impl ConversionPlan {
    pub fn needs_conversion(&self) -> bool {
        !matches!(self.video, VideoAction::Copy)
            || !matches!(self.audio, AudioAction::Copy)
        }
}
