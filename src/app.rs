use ratatui::{
    Frame,
    style::Color,
    widgets::{List, ListState},
};
use std::{
    fs::{self},
    path::PathBuf,
};
use tracing;

pub struct App {
    state: ListState,
    tracks: Vec<PathBuf>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            state: ListState::default().with_selected(Some(0)),
            tracks: Vec::new(),
        };

        app.read_music();

        app
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let items: Vec<&str> = self
            .tracks
            .iter()
            .map(|item| item.file_name().unwrap().to_str().unwrap())
            .collect();
        let list = List::new(items)
            .style(Color::LightBlue)
            .highlight_symbol(">");

        frame.render_stateful_widget(list, frame.area(), &mut self.state);
    }

    pub fn previous(&mut self) {
        self.state.select_previous();
    }

    pub fn next(&mut self) {
        self.state.select_next();
    }

    pub fn read_music(&mut self) {
        let dir = fs::read_dir("./music");
        tracing::info!("{:?}", std::env::current_dir());
        if dir.is_ok() {
            for entry in dir.unwrap() {
                if entry.is_ok() {
                    let file = entry.unwrap();

                    self.tracks.push(file.path())
                }
            }
        }
    }
}
