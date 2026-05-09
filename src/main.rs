mod app;

use std::fs::File;

use app::App;
use crossterm::event::{self, Event, KeyCode};
use ratatui::DefaultTerminal;
use tracing_subscriber::fmt;

fn main() -> color_eyre::Result<()> {
    init_logging();

    color_eyre::install()?;
    ratatui::run(run)?;
    Ok(())
}

fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| app.render(frame))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break Ok(()),
                KeyCode::Up => app.previous(),
                KeyCode::Down => app.next(),
                KeyCode::Char(' ') => app.play(),
                _ => {}
            }
        }
    }
}

fn init_logging() {
    let file = File::create("debug.log").unwrap();
    fmt().with_writer(file).with_ansi(false).init();
}
