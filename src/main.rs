mod media;
mod probe;
mod converter;
mod planner;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::fs;

use crate::planner::PlanningPreferences;

#[derive(Parser, Debug)]
#[command(name = "codecbridge")]
#[command(about = "Media compatibility helper")]

struct Args {
    /// Media file to inspect
    inputs: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    for input in &args.inputs {
        if input.is_dir() {
            for entry in fs::read_dir(input)? {
                let path = entry?.path();

                if path.is_file() && is_media_file(&path) {
                    if let Err(error) = process_file(&path) {
                        eprintln!("Failed: {}\n{error:#}", path.display());
                    }
                }
            }
        } else {
            if let Err(error) = process_file(input) {
                eprintln!("Failed: {}\n{error:#}", input.display());
            }
        }
    }

    Ok(())
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

    if plan.needs_conversion() {
        let output = converter::convert_file(input, &plan)?;
        println!("Converted file: {}", output.display());
    } else {
        println!("No conversion needed.");
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
