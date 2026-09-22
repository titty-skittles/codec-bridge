use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use anyhow::Result;
use std::thread;

use crate::media::MediaFile;
use crate::planner::{self, ConversionPlan, PlanningPreferences, VideoPreference};
use crate::{is_media_file, probe};
use crate::output::{
    self,
    CollisionPolicy,
    OutputDirectory,
    OutputPreferences,
    OutputResolution,
};

#[derive(Debug)]
pub enum JobStatus {
    Ready,
    NotNeeded,
    Converting,
    Complete,
    Skipped(String),
    Failed(String),
}

#[derive(Debug, Clone, Copy)]
pub enum Page {
    Queue,
    Preferences,
    Progress,
}

#[derive(Debug)]
pub struct Job {
    pub path: PathBuf,
    pub media: MediaFile,
    pub plan: ConversionPlan,
    pub enabled: bool,
    pub status: JobStatus,
    pub progress: f64,
}

#[derive(Debug)]
pub struct App {
    pub should_quit: bool,
    pub jobs: Vec<Job>,
    pub selected: usize,
    pub worker_tx: Sender<WorkerMessage>,
    pub worker_rx: Receiver<WorkerMessage>,
    pub page: Page,
    pub default_preferences: PlanningPreferences,
    pub worker_running: bool,
    pub output_preferences: OutputPreferences,
    pub adding_path: bool,
    pub path_input: String,
}

#[derive(Debug)]
enum WorkerMessage {
    Started(usize),
    Skipped(usize, String),
    Complete(usize),
    Failed(usize, String),
    Progress(usize, f64),
    QueueComplete,
}

impl JobStatus {
    pub fn description(&self) -> &str {
        match self {
            JobStatus::Ready => "Ready",
            JobStatus::NotNeeded => "Not needed",
            JobStatus::Converting => "Converting",
            JobStatus::Complete => "Complete",
            JobStatus::Failed(_) => "Failed",
            JobStatus::Skipped(_) => "Skipped",
        }
    }
}

impl App {
    pub fn new() -> Self {
        let (worker_tx, worker_rx) = mpsc::channel();

        Self {
            should_quit: false,
            jobs: Vec::new(),
            selected: 0,
            worker_tx,
            worker_rx,
            page: Page::Preferences,
            default_preferences: PlanningPreferences { video: VideoPreference::Preserve, },
            worker_running: false,
            output_preferences: OutputPreferences::default(),
            adding_path : false,
            path_input: String::new(),
        }
    }

    pub fn begin_add_path(&mut self) {
        self.adding_path = true;
        self.path_input.clear();
    }

    pub fn cancel_add_path(&mut self) {
        self.adding_path = false;
        self.path_input.clear();
    }

    pub fn push_path_character(&mut self, character: char) {
        self.path_input.push(character);
    }

    pub fn pop_path_character(&mut self) {
        self.path_input.pop();
    }

    pub fn submit_path(&mut self) {
        let normalized = normalize_path_input(&self.path_input);
        let input = PathBuf::from(normalized);

        self.adding_path = false;
        self.path_input.clear();

        if input.is_file() {
            let _ = self.add_path(input);
            return;
        }

        if input.is_dir() {
            let Ok(entries) = std::fs::read_dir(input) else {
                return;
            };
            
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_file() && is_media_file(&path) {
                    let _ = self.add_path(path);
                }
            }
        }
    }

    pub fn cycle_output_directory(&mut self) {
        self.output_preferences.directory =
            match &self.output_preferences.directory {
                OutputDirectory::SameAsSource => {
                    OutputDirectory::Subdirectory(
                        "codecbridge".to_string()
                    )
                }

                OutputDirectory::Subdirectory(_) => {
                    OutputDirectory::SameAsSource
                }

                OutputDirectory::Custom(_) => {
                    OutputDirectory::SameAsSource
                }
            };
    }

    pub fn cycle_collision_policy(&mut self) {
        self.output_preferences.collision = 
            match self.output_preferences.collision {
                CollisionPolicy::Skip => CollisionPolicy::Rename,
                CollisionPolicy::Rename => CollisionPolicy::Overwrite,
                CollisionPolicy::Overwrite => CollisionPolicy::Skip,
            };
    }
    pub fn active_job(&self) -> Option<&Job> {
        self.jobs
            .iter()
            .find(|job| matches!(job.status, JobStatus::Converting))
    }

    pub fn next_page(&mut self) {
        self.page = match self.page {
            Page::Queue => Page::Preferences,
            Page::Preferences => Page::Progress,
            Page::Progress => Page::Queue,
        };
    }

    pub fn previous_page(&mut self) {
        self.page = match self.page {
            Page::Queue => Page::Progress,
            Page::Preferences => Page::Queue,
            Page::Progress => Page::Preferences,
        };
    }

    pub fn toggle_default_video_preference(&mut self) {
        self.default_preferences.video =
            match self.default_preferences.video {
                VideoPreference::Preserve => VideoPreference::ForceDnxhr,
                VideoPreference::ForceDnxhr => VideoPreference::Preserve,
            };
    }

    pub fn add_path(&mut self, path: PathBuf) -> Result<()> {
        let media = probe::probe_file(&path)?;

        let plan = planner::plan_conversion(
            &media, 
            &self.default_preferences
        );
        let enabled = plan.needs_conversion();

        let status = if enabled {
            JobStatus::Ready
        } else {
            JobStatus::NotNeeded
        };

        self.jobs.push(Job {
            path,
            media,
            plan,
            enabled,
            status,
            progress: 0.0,
        });

        Ok(())
    }

    pub fn toggle_selected(&mut self) {
        if let Some(job) = self.jobs.get_mut(self.selected) {
            job.enabled = !job.enabled;
        }
    }

    pub fn next(&mut self) {
        if self.jobs.is_empty() {
            return;
        }

        self.selected = (self.selected + 1) % self.jobs.len();
    }

    pub fn previous(&mut self) {
        if self.jobs.is_empty() {
            return;
        }

        if self.selected == 0 {
            self.selected = self.jobs.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    pub fn handle_worker_messages(&mut self) {
        while let Ok(message) = self.worker_rx.try_recv() {
            match message {
                WorkerMessage::Started(index) => {
                    if let Some(job) = self.jobs.get_mut(index) {
                        job.status = JobStatus::Converting;
                        job.progress = 0.0;
                    }
                }

                WorkerMessage::Skipped(index, reason) => {
                    if let Some(job) = self.jobs.get_mut(index) {
                        job.status = JobStatus::Skipped(reason);
                    }
                }

                WorkerMessage::Progress(index, percent) => {
                    if let Some(job) = self.jobs.get_mut(index) {
                        job.progress = percent;
                    }
                }

                WorkerMessage::Complete(index) => {
                    if let Some(job) = self.jobs.get_mut(index) {
                        job.status = JobStatus::Complete;
                    }
                }

                WorkerMessage::Failed(index, error) => {
                    if let Some(job) = self.jobs.get_mut(index) {
                        job.status = JobStatus::Failed(error);
                    }
                }
                WorkerMessage::QueueComplete => {
                    self.worker_running = false;
                }
            }
        }
    }

    pub fn toggle_video_mode(&mut self) {
        let Some(job) = self.jobs.get_mut(self.selected) else {
            return;
        };

        let preference = match job.plan.video {
            crate::planner::VideoAction::Copy => {
                crate::planner::VideoPreference::ForceDnxhr
            }

            crate::planner::VideoAction::TranscodeDnxhr => {
                crate::planner::VideoPreference::Preserve
            }
        };

        let preferences = crate::planner::PlanningPreferences {
            video: preference,
        };

        job.plan = crate::planner::plan_conversion(
            &job.media,
            &preferences,
        );

        job.enabled = job.plan.needs_conversion();

        job.status = if job.enabled {
            JobStatus::Ready
        } else {
            JobStatus::NotNeeded
        };
    }

    pub fn remove_selected(&mut self) {
        if self.jobs.is_empty() {
            return;
        }

        self.jobs.remove(self.selected);

        if self.jobs.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.jobs.len() {
            self.selected = self.jobs.len() - 1;
        }
    }

    pub fn run_conversions(&mut self) {
        if self.worker_running {
            return;
        }

        let jobs: Vec<(usize, std::path::PathBuf, ConversionPlan, Option<f64>)> = self
            .jobs
            .iter()
            .enumerate()
            .filter(|(_, job)| job.enabled && job.plan.needs_conversion())
            .map(|(index,job)| {
                (
                    index,
                    job.path.clone(),
                    job.plan.clone(),
                    job.media.duration_seconds,
                )
            })
            .collect();
        if jobs.is_empty() {
            return;
        }

        self.worker_running = true;

        let tx = self.worker_tx.clone();
        let output_preferences = self.output_preferences.clone();

        thread::spawn(move || {
            for (index, path, plan, duration_seconds) in jobs {
                let _ = tx.send(WorkerMessage::Started(index));

                let output = match output::resolve_output_path(
                    &path,
                    &output_preferences,
                ) {
                    Ok(OutputResolution::Path(path)) => path,

                    Ok(OutputResolution::Skip) => {
                        let _ = tx.send(WorkerMessage::Skipped(
                                index,
                                "Output already  exists".to_string(),
                        ));
                        continue;
                    }

                    Err(error) => {
                        let _ = tx.send(WorkerMessage::Failed(
                                index,
                                error.to_string(),
                        ));
                        continue;
                    }
                };

                let overwrite = matches!(
                    output_preferences.collision,
                    CollisionPolicy::Overwrite
                );

                match crate::converter::convert_file(
                    &path,
                    &output,
                    overwrite,
                    &plan,
                    duration_seconds,
                    |percent| {
                        let _ = tx.send(WorkerMessage::Progress(index, percent));
                    },
                ) {
                    Ok(_) => {
                        let _ = tx.send(WorkerMessage::Complete(index));
                    }

                    Err(error) => {
                        let _ = tx.send(WorkerMessage::Failed(
                                index,
                                error.to_string(), 
                                ));
                    }
                }
            }
            let _ = tx.send(WorkerMessage::QueueComplete);
        });
    }
}

fn normalize_path_input(input: &str) -> String {
    let trimmed = input.trim();

    let trimmed = trimmed
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .unwrap_or(trimmed);

    let trimmed = trimmed
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(trimmed);

    trimmed.replace("\\ ", " ")
}
