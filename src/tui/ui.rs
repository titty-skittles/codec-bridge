use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::app::{App, JobStatus};

pub fn draw(frame: &mut Frame, app: &App) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(areas[0]);

    let items: Vec<ListItem> = app
        .jobs
        .iter()
        .enumerate()
        .map(|(index, job)| {
            let marker = if index == app.selected { "> " } else { "  " };
            let enabled = if job.enabled { "[x]" } else { "[ ]" };

            let name = job
                .path
                .file_name()
                .map(|name| name.to_string_lossy())
                .unwrap_or_else(|| job.path.to_string_lossy());

            ListItem::new(format!(
                    "{marker} {enabled} {name}  {}",
                    job.status.description()))
        })
        .collect();

    let file_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Files "),
        );

    let details_text = if let Some(job) = app.jobs.get(app.selected) {

        let status = match &job.status {
            JobStatus::Failed(error) => {
                format!("Status: Failed\nError: {error}")
            }

            _ => {
                format!("Status: {}", job.status.description())
            }
        };

        let video = job
            .media
            .first_video()
            .map(|video| {
                format!(
                    "{}\n{}x{}\nPixel format: {}\nPlan: {}",
                    video.codec,
                    video.width,
                    video.height,
                    video.pixel_format.as_deref().unwrap_or("unknown"),
                    job.plan.video.description(),
                )
            })
            .unwrap_or_else(|| "No video stream".to_string());

        let audio = job
            .media
            .first_audio()
            .map(|audio| {
                format!(
                    "{}\nSample rate: {}\nChannels: {}\nPlan: {}",
                    audio.codec,
                    audio
                        .sample_rate
                        .map(|rate| format!("{rate} Hz"))
                        .unwrap_or_else(|| "unknown".to_string()),
                    audio
                        .channels
                        .map(|channels| channels.to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    job.plan.audio.description(),
                )
            })
            .unwrap_or_else(|| "No audio stream".to_string());

        format!(
            "{}\n\n{}\n\nVideo\n{}\n\nAudio\n{}",
            job.path.display(),
            status,
            video,
            audio,
        )
    } else {
        "No files loaded".to_string()
    };

    let details = Paragraph::new(details_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Details "),
        );

    let footer = Paragraph::new("↑/k up   ↓/j down   <Space> select   v video mode   <Enter> run   q quit");

    frame.render_widget(file_list, columns[0]);
    frame.render_widget(details, columns[1]);
    frame.render_widget(footer, areas[1]);
}
