use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Paragraph,
    Frame,
};

use super::app::{App, Page};
use super::pages;

pub fn draw(frame: &mut Frame, app: &App) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    draw_header(frame, app, areas[0]);

    match app.page {
        Page::Queue => pages::queue::draw(frame, app, areas[1]),
        Page::Preferences => pages::preferences::draw(frame, app, areas[1]),
        Page::Progress => pages::progress::draw(frame, app, areas[1]),
    }

    draw_footer(frame, app, areas[2]);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let  preferences = if matches!(app.page, Page::Preferences) {
        "[1 Preferences]"
    } else {
        " 1 Preferences "
    };

    let queue = if matches!(app.page, Page::Queue) {
        "[2 Queue]"
    } else {
        " 2 Queue "
    };

    let progress = if matches!(app.page, Page::Progress) {
        "[3 Progress]"
    } else {
        " 3 Progress "
    };

    let header = Paragraph::new(format!(
        "{preferences}  {queue}  {progress}"
    ));

    frame.render_widget(header, area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let text = if app.adding_path {
        format!(
            "Add path: {}█ | Enter confirm | Esc cancel",
            app.path_input
        )
    } else {
        match app.page {
            Page::Queue => {
                "↑↓/jk move | <Space> toggle | a add file/path | x remove | v video | <Enter> run | <Tab> next page | q quit"
                    .to_string()
            }

            Page::Preferences => {
                "v video default | o output location | c collision policy | <Tab> next page | q quit"
                    .to_string()
            }

            Page::Progress => {
                "1 Queue | 2 Preferences | 3 Progress | <Tab> next page | q quit"
                    .to_string()
            }
        }
    };

    frame.render_widget(Paragraph::new(text), area);
}

