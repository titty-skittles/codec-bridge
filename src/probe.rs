use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;
use crate::media::{AudioStream, MediaFile, MediaStream, VideoStream};


#[derive(Debug, Deserialize)]
struct ProbeOutput {
    streams: Vec<ProbeStream>,
    format: Option<ProbeFormat>,
}

#[derive(Debug, Deserialize)]
struct ProbeFormat {
    duration: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProbeStream {
    codec_name: Option<String>,
    codec_type: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    pix_fmt: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u32>,
}

pub fn probe_file(path: &Path) -> Result<MediaFile> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .context("Failed to run ffprobe")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffprobe failed:\n{stderr}");
    }

    let probe: ProbeOutput = 
        serde_json::from_slice(&output.stdout)
            .context("ffprobe output was not valid UTF-8")?;

    let duration_seconds = probe
        .format
        .as_ref()
        .and_then(|format| format.duration.as_ref())
        .and_then(|duration| duration.parse::<f64>().ok());


    let streams = probe
        .streams
        .into_iter()
        .map(|stream| {
            match stream.codec_type.as_deref() {
                Some("video") => MediaStream::Video(VideoStream {
                    codec: stream.codec_name.unwrap_or_else(|| "unknown".to_string()),
                    width: stream.width.unwrap_or(0),
                    height: stream.height.unwrap_or(0),
                    pixel_format: stream.pix_fmt,
                }),

                Some("audio") => MediaStream::Audio(AudioStream {
                    codec: stream.codec_name.unwrap_or_else(|| "unknown".to_string()),
                    sample_rate: stream
                        .sample_rate
                        .and_then(|rate| rate.parse::<u32>().ok()),
                    channels: stream.channels,
                }),

                _ => MediaStream::Other,
            }
        })
    .collect();

    Ok(MediaFile { 
        streams,
        duration_seconds,
    })
}
