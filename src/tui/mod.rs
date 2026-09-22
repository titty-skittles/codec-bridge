pub mod app;
pub mod ui;
pub mod pages;

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
                if app.adding_path {
                    match key.code {
                        KeyCode::Enter => app.submit_path(),

                        KeyCode::Esc => app.cancel_add_path(),

                        KeyCode:: Backspace => app.pop_path_character(),

                        KeyCode::Char(character) => {
                            app.push_path_character(character);
                        }

                        _ => {}
                    }

                    continue; 
                }

                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Char('a') => {
                        if matches!(app.page, app::Page::Queue) {
                            app.begin_add_path();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Char(' ') => app.toggle_selected(),
                    KeyCode::Enter => {
                        if matches!(app.page, app::Page::Queue) {
                            app.run_conversions();

                            if app.worker_running {
                                app.page = app::Page::Progress;
                            }
                        }
                    }
                    KeyCode::Char('o') => {
                        if matches!(app.page, app::Page::Preferences) {
                            app.cycle_output_directory();
                        }
                    }
                    KeyCode::Char('v') => {
                        match app.page {
                            app::Page::Queue => app.toggle_video_mode(),
                            app::Page::Preferences => app.toggle_default_video_preference(),
                            app::Page::Progress => {}
                        }
                    }
                    KeyCode::Char('c') => {
                        if matches!(app.page, app::Page::Preferences) {
                            app.cycle_collision_policy();
                        }
                    }
                    KeyCode::Char('x') => {
                        if matches!(app.page, app::Page::Queue) {
                            app.remove_selected();
                        }
                    }
                    KeyCode::Tab => app.next_page(),
                    KeyCode::BackTab => app.previous_page(),
                    KeyCode::Char('2') => app.page = app::Page::Queue,
                    KeyCode::Char('1') => app.page = app::Page::Preferences,
                    KeyCode::Char('3') => app.page = app::Page::Progress,
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
