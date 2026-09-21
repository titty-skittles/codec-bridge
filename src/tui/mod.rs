pub mod app;
pub mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub fn run(paths: Vec<std::path::PathBuf>) -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    for path in paths {
        if let Err(error) = app.add_path(path.clone()) {
            eprintln!("Failed to load {}: {error:#}", path.display());
        }
    }

    while !app.should_quit {
        app.handle_worker_messages();

        terminal.draw(|frame| {
            ui::draw(frame, &app);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Char(' ') => app.toggle_selected(),
                    KeyCode::Enter => app.run_conversions(),
                    KeyCode::Char('v') => app.toggle_video_mode(),
                    _ => {}
                }
            }
        }
    }

       disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
