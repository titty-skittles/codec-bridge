use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::planner::VideoPreference;

pub fn draw(frame: &mut Frame, app: &App) {
    let video = match app.default_preferences.video {
        VideoPreference::Preserve => "Preserve compatible video",
        VideoPreference::ForceDnxhr => "Force DNxHR",
    };

    let widget = Paragraph::new(format!(
        "Default video plan\n\n{video}\n\nPress v to toggle"
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Preferences "),
    );

    frame.render_widget(widget, frame.area());
}
