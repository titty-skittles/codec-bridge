mod media;
mod probe;

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

    let probe = probe::probe_file(&args.input)?;
    println!("{probe:#?}");

    Ok(())
}
