use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;

pub fn draw(frame: &mut Frame, _app: &App) {
    let widget = Paragraph::new(
        "Preferences\n\nDefault video plan: Preserve\nDefault audio plan: PCM when required"
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Preferences "),
    );

    frame.render_widget(widget, frame.area());
}
