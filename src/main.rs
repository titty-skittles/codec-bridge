mod media;
mod probe;
mod converter;
mod planner;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use planner::ConversionPlan;

#[derive(Parser, Debug)]
#[command(name = "codecbridge")]
#[command(about = "Media compatibility helper")]
struct Args {
    /// Media file to inspect
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let media = probe::probe_file(&args.input)?;
    println!("{media:#?}");

    let plan = planner::plan_conversion(&media);

    match plan {
        ConversionPlan::ConvertAudioToPcm => {
            let output = converter::convert_audio_to_pcm(&args.input)?;
            println!("Converterd file: {}", output.display());
        }

        ConversionPlan::NoChange => {
            println!("No conversion needed.");
        }
    }


    Ok(())
}
