use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::planner::VideoPreference;
use crate::output::CollisionPolicy;

pub fn draw(frame: &mut Frame, app: &App) {
    let video = match app.default_preferences.video {
        VideoPreference::Preserve => "Preserve compatible video",
        VideoPreference::ForceDnxhr => "Force DNxHR",
    };

    let collision = match app.output_preferences.collision {
        CollisionPolicy::Skip => "Skip existing output",
        CollisionPolicy::Rename => "Create numbered copy",
        CollisionPolicy::Overwrite => "Overwrite existing output",
    };

    let widget = Paragraph::new(format!(
        "Default video plan\n\
        {video}\n\n\
        Output\n\
        Location: Same as source\n\
        Filename: {{stem}}_codecbrdige.mov\n\
        Existing ouptut: {collision}\n\n\
        v Toggle video mode\n\
        c Change collision policy"
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Preferences "),
    );

    frame.render_widget(widget, frame.area());
}
