mod media;
mod probe;
mod converter;
mod planner;
mod tui;
mod output;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "codecbridge")]
#[command(about = "Media compatibility helper")]

struct Args {
    /// Media file to inspect
    inputs: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut paths = Vec::new();

    if args.inputs.is_empty() {
        let current_dir = std::env::current_dir()?;

        for entry in fs::read_dir(current_dir)? {
            let path = entry?.path();

            if path.is_file() && is_media_file(&path) {
                paths.push(path);
            }
        }
    } else {
        for input in &args.inputs {
            if input.is_dir() {
                for entry in fs::read_dir(input)? {
                    let path = entry?.path();

                    if path.is_file() && is_media_file(&path) {
                        paths.push(path);
                    }
                }
            } else if input.is_file() {
                paths.push(input.clone());
            } else {
                eprintln!("Input not found: {}", input.display());
            }
        }
    }

    tui::run(paths)
}

fn process_file(input: &std::path::Path) -> Result<()> {
    println!("Processing: {}", input.display());
    let media = probe::probe_file(input)?;

    let preferences = planner::PlanningPreferences {
        video: planner::VideoPreference::Preserve,
    };

    let plan = planner::plan_conversion(&media, &preferences);

    if let Some(video) = media.first_video() {
        println!(
            " video: {} → {}",
            video.codec,
            plan.video.description()
        );
    }

    if let Some(audio) = media.first_audio() {
        println!(
            " Audio: {} → {}",
            audio.codec,
            plan.audio.description()
        );
    }

    Ok(())
}

fn is_media_file(path: &std::path::Path) -> bool {
    let Some(extension) = path.extension() else {
        return false;
    };

    matches!(
        extension.to_string_lossy().to_lowercase().as_str(),
        "mp4"
            | "mov"
            | "mkv"
            | "m4v"
            | "avi"
            | "webm"
            | "mts"
            | "m2ts"
            | "mpg"
            | "mpeg"
    )
}          
