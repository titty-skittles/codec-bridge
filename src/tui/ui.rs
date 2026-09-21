use ratatui::{
    Frame,
};

use super::app::{App, Page};
use super::pages;

pub fn draw(frame: &mut Frame, app: &App) {
    match app.page {
        Page::Queue => pages::queue::draw(frame, app),
        Page::Preferences => pages::preferences::draw(frame, app),
        Page::Progress => pages::progress::draw(frame, app),
    }
}



