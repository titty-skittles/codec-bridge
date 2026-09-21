use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::app::{App, JobStatus};

pub fn draw(frame: &mut Frame, app: &App) {
   let total = app.jobs.iter().filter(|job| job.enabled).count();

    let complete = app
        .jobs
        .iter()
        .filter(|job| {
            job.enabled && matches!(job.status, JobStatus::Complete)
        })
        .count();

    let failed = app
        .jobs
        .iter()
        .filter(|job| {
            job.enabled && matches!(job.status, JobStatus::Failed(_))
        })
        .count();

    let converting = app
        .jobs
        .iter()
        .filter(|job| {
            job.enabled && matches!(job.status, JobStatus::Converting)
        })
        .count();

    let pending = total
        .saturating_sub(complete + failed + converting); 


    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let total = app.jobs.iter().filter(|job| job.enabled).count();

    let complete = app
        .jobs
        .iter()
        .filter(|job| {
            job.enabled && matches!(job.status, JobStatus::Complete)
        })
        .count();

    let summary = Paragraph::new(format!(
        "Completed: {complete} / {total}"
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Queue "),
    );

    frame.render_widget(summary, areas[0]);

    if let Some(job) = app.active_job() {
        let name = job
            .path
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_else(|| job.path.to_string_lossy());

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Converting: {name} ")),
            )
            .percent(job.progress.round() as u16);

        frame.render_widget(gauge, areas[1]);
    } else {
        let idle = Paragraph::new("No conversion currently running")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Current "),
            );

        frame.render_widget(idle, areas[1]);
    }

    let items: Vec<ListItem> = app
        .jobs
        .iter()
        .filter(|job| job.enabled)
        .map(|job| {
            let symbol = match job.status {
                JobStatus::Ready => "•",
                JobStatus::NotNeeded => "-",
                JobStatus::Converting => "▶",
                JobStatus::Complete => "✓",
                JobStatus::Failed(_) => "!",
            };

            let name = job
                .path
                .file_name()
                .map(|name| name.to_string_lossy())
                .unwrap_or_else(|| job.path.to_string_lossy());

            ListItem::new(format!(
                "{symbol} {name}  {}",
                job.status.description()
            ))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Jobs "),
        );

    frame.render_widget(list, areas[2]);

    let footer =
        Paragraph::new("1 Queue   2 Preferences   3 Progress   Tab next   q quit");

    frame.render_widget(footer, areas[3]);

}
