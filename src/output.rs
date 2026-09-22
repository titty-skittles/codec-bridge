use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum OutputDirectory {
    SameAsSource,
    Subdirectory(String),
    Custom(PathBuf),
}

#[derive(Debug, Clone, Copy)]
pub enum CollisionPolicy {
    Skip,
    Rename,
    Overwrite,
}

#[derive(Debug, Clone)]
pub struct OutputPreferences {
    pub directory: OutputDirectory,
    pub suffix: String,
    pub collision: CollisionPolicy,
}

impl Default for OutputPreferences {
    fn default() -> Self {
        Self {
            directory: OutputDirectory::Subdirectory(
                "codecbrdige".to_string(),
            ),
            suffix: "_cb".to_string(),
            collision: CollisionPolicy::Skip,
        }
    }
}

pub enum OutputResolution {
    Path(PathBuf),
    Skip,
}

pub fn resolve_output_path(
    input: &Path,
    preferences: &OutputPreferences,
) -> Result<OutputResolution> {
    let stem = input
        .file_stem()
        .context("Input file has no filename")?
        .to_string_lossy();

    let directory = match &preferences.directory {
        OutputDirectory::SameAsSource => input
            .parent()
            .context("Input file has no parent directory")?
            .to_path_buf(),

        OutputDirectory::Subdirectory(name) => {
            let parent = input
                .parent()
                .context("Input file has no parent directory")?;

            let directory = parent.join(name);

            std::fs::create_dir_all(&directory)
                .context("Failed to create output directory")?;

            directory
        }

        OutputDirectory::Custom(path) => {
            std::fs::create_dir_all(path)
                .context("Failed to create output directory")?;

            path.clone()
        }
    };

    let filename = format!("{stem}{}.mov", preferences.suffix);
    let output = directory.join(filename);

    if !output.exists() {
        return Ok(OutputResolution::Path(output));
    }

    match preferences.collision {
        CollisionPolicy::Skip => Ok(OutputResolution::Skip),

        CollisionPolicy::Overwrite => {
            Ok(OutputResolution::Path(output))
        }

        CollisionPolicy::Rename => {
            let mut number = 2;

            loop {
                let filename =
                    format!("{stem}{}_{}.mov", preferences.suffix, number);

                let candidate = directory.join(filename);

                if !candidate.exists() {
                    return Ok(OutputResolution::Path(candidate));
                }

                number += 1;
            }
        }
    }
}
