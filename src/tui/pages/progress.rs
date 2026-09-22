use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::app::{App, JobStatus};

pub fn draw(frame: &mut Frame,
    app: &App,
    area: Rect,
) {
   let total = app.jobs.iter().filter(|job| job.enabled).count();

    let complete = app
        .jobs
        .iter()
        .filter(|job| {
            job.enabled && matches!(job.status, JobStatus::Complete)
        })
        .count();

    let skipped = app
        .jobs
        .iter()
        .filter(|job| {
            job.enabled && matches!(job.status, JobStatus::Skipped(_))
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
        .saturating_sub(complete + skipped + failed + converting); 


    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    let total = app.jobs.iter().filter(|job| job.enabled).count();

    let complete = app
        .jobs
        .iter()
        .filter(|job| {
            job.enabled && matches!(job.status, JobStatus::Complete)
        })
        .count();

   let summary = Paragraph::new(format!(
        "Completed: {complete}/{total}    Pending: {pending}    Skipped: {skipped}    Failed: {failed}"
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Queue Progress "),
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
            let name = job
                .path
                .file_name()
                .map(|name| name.to_string_lossy())
                .unwrap_or_else(|| job.path.to_string_lossy());

            let text = match &job.status {
                JobStatus::Ready => {
                    format!("• {name}  Pending")
                }

                JobStatus::NotNeeded => {
                    format!("- {name}  Not needed")
                }

                JobStatus::Skipped(reason) => {
                    format!("- {name} SKipped: {reason}")
                }

                JobStatus::Converting => {
                    format!("▶ {name}  {:.0}%", job.progress)
                }

                JobStatus::Complete => {
                    format!("✓ {name}  Complete")
                }

                JobStatus::Failed(error) => {
                    format!("! {name}  Failed: {error}")
                }
            };

            ListItem::new(text)
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
