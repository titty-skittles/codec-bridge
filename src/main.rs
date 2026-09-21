mod media;
mod probe;
mod converter;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "codecbridge")]
#[command(about = "Media compatibility helper")]
struct Args {
    /// Media file to inspect
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let output = converter::convert_audio_to_pcm(&args.input)?;
    println!("Converterd file: {}", output.display());



    Ok(())
}
