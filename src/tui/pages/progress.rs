use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;

pub fn draw(frame: &mut Frame, _app: &App) {
    let widget = Paragraph::new("No active conversions")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Progress "),
        );

    frame.render_widget(widget, frame.area());
}
