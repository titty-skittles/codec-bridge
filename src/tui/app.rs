use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use anyhow::Result;
use std::thread;

use crate::media::MediaFile;
use crate::planner::{self, ConversionPlan, PlanningPreferences, VideoPreference};
use crate::probe;

#[derive(Debug)]
pub enum JobStatus {
    Ready,
    NotNeeded,
    Converting,
    Complete,
    Failed(String),
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
}

#[derive(Debug)]
enum WorkerMessage {
    Started(usize),
    Complete(usize),
    Failed(usize, String),
    Progress(usize, f64),
}

impl JobStatus {
    pub fn description(&self) -> &str {
        match self {
            JobStatus::Ready => "Ready",
            JobStatus::NotNeeded => "Not needed",
            JobStatus::Converting => "Converting",
            JobStatus::Complete => "Complete",
            JobStatus::Failed(_) => "Failed",
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
        }
    }

    pub fn add_path(&mut self, path: PathBuf) -> Result<()> {
        let media = probe::probe_file(&path)?;

        let preferences = PlanningPreferences {
            video: VideoPreference::Preserve,
        };

        let plan = planner::plan_conversion(&media, &preferences);
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

    pub fn run_conversions(&mut self) {
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

        let tx = self.worker_tx.clone();

        thread::spawn(move || {
            for (index, path, plan, duration_seconds) in jobs {
                let _ = tx.send(WorkerMessage::Started(index));

                match crate::converter::convert_file(
                    &path,
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
        });
    }
}
