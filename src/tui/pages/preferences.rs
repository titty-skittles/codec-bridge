use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::planner::VideoPreference;
use crate::output::{
    CollisionPolicy,
    OutputDirectory,
};

pub fn draw(frame: &mut Frame, 
    app: &App,
    area: Rect,
) {
    let video = match app.default_preferences.video {
        VideoPreference::Preserve => "Preserve compatible video",
        VideoPreference::ForceDnxhr => "Force DNxHR",
    };

    let collision = match app.output_preferences.collision {
        CollisionPolicy::Skip => "Skip existing output",
        CollisionPolicy::Rename => "Create numbered copy",
        CollisionPolicy::Overwrite => "Overwrite existing output",
    };

    let output_location = match &app.output_preferences.directory {
        OutputDirectory::SameAsSource => {
            "Same as source".to_string()
        }

        OutputDirectory::Subdirectory(name) => {
            format!("Subdirectory: {name}")
        }

        OutputDirectory::Custom(path) => {
            format!("Custom: {}", path.display())
        }
    };

    let widget = Paragraph::new(format!(
        "Default video plan\n\
        {video}\n\n\
        Output\n\
        Location: {output_location}\n\
        Filename: {{stem}}_codecbrdige.mov\n\
        Existing ouptut: {collision}\n\n\
        v Toggle video mode\n\
        o Change output location\n\
        c Change collision policy"
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Preferences "),
    );

    frame.render_widget(widget, area);
}
