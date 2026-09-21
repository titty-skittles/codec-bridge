use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use crate::planner::{AudioAction, ConversionPlan, VideoAction};
use std::io::{BufRead, BufReader};

pub fn convert_file<F>(
    input: &Path,
    plan: &ConversionPlan,
    duration_seconds: Option<f64>,
    mut on_progress: F,
) -> Result<PathBuf> 
where
    F:FnMut(f64),
{
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
            "-n",
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

    command
        .args([
            "-progress",
            "pipe:1",
            "-nostats",
        ]);

    let mut child = command
        .arg(&output)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to run ffmpeg")?;

    let stdout = child
        .stdout
        .take()
        .context("Failed to capture ffmpeg progress")?;

    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line?;

        if let Some(value) = line.strip_prefix("out_time_us=") {
            if let (Ok(microseconds), Some(duration)) = 
                (value.parse::<f64>(), duration_seconds)
            {
                if duration > 0.0 {
                    let seconds = microseconds / 1_000_000.0;
                    let percent = (seconds / duration * 100.0).clamp(0.0,100.0);

                    on_progress(percent);
                }
            }
        }
    }

    let status = child
        .wait()
        .context("Failed to wait for ffmpeg")?;

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
