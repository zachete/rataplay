use crate::app::audio_player::AudioPlayer;
use crate::app::library::Library;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::{Block, List, ListState},
};
use std::path::PathBuf;

struct State {
    current_track: Option<PathBuf>,
    albums_list_state: ListState,
    track_list_state: ListState,
}

pub struct App {
    audio_player: AudioPlayer,
    state: State,
    library: Library,
    prev_selected: Option<usize>,
    tracks_selected: bool,
}

impl App {
    pub fn new() -> Self {
        let audio_player = AudioPlayer::new();
        let state = State {
            current_track: None,
            albums_list_state: ListState::default().with_selected(Some(0)),
            track_list_state: ListState::default().with_selected(Some(0)),
        };
        let mut library = Library::new();
        library.load();

        let app = Self {
            audio_player,
            state,
            library,
            prev_selected: None,
            tracks_selected: false,
        };

        app
    }

    pub fn previous(&mut self) {
        if self.tracks_selected {
            self.state.track_list_state.select_previous();
        } else {
            self.state.albums_list_state.select_previous();
            self.state.track_list_state.select_first();
        }

        self.set_current_track();
    }

    pub fn next(&mut self) {
        if self.tracks_selected {
            self.state.track_list_state.select_next();
        } else {
            self.state.albums_list_state.select_next();
            self.state.track_list_state.select_first();
        }

        self.set_current_track();
    }

    fn set_current_track(&mut self) {
        let track = self.library.get_track(
            self.state.albums_list_state.selected().unwrap(),
            self.state.track_list_state.selected().unwrap(),
        );
        self.state.current_track = Some(track.clone());
    }

    pub fn focus_albums(&mut self) {
        self.tracks_selected = false
    }

    pub fn focus_tracks(&mut self) {
        self.tracks_selected = true
    }

    pub fn play(&mut self) {
        let selected = self.state.track_list_state.selected().unwrap();

        match self.prev_selected {
            Some(val) => {
                if selected != val {
                    self.play_selected_file();
                } else {
                    self.audio_player.play();
                }
            }
            None => {
                self.play_selected_file();
            }
        }

        self.prev_selected = Some(selected);
    }

    fn play_selected_file(&mut self) {
        let file_path_buf = self.state.current_track.as_ref().unwrap();
        self.audio_player.set_current_track(file_path_buf);
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let layout =
            Layout::horizontal([Constraint::Length(30), Constraint::Length(30)]).spacing(1);

        let [dirs, files] = frame.area().layout(&layout);
        self.render_albums(frame, dirs);
        self.render_tracks(frame, files);
    }

    pub fn render_albums(&mut self, frame: &mut Frame, area: Rect) {
        let style = if self.tracks_selected {
            Color::LightBlue
        } else {
            Color::White
        };
        let block = Block::bordered().border_style(style).title("Albums");
        let items: Vec<String> = self
            .library
            .get_albums()
            .iter()
            .map(|item| item.borrow().title.clone())
            .collect();
        let list = List::new(items).highlight_symbol(">").block(block);

        frame.render_stateful_widget(list, area, &mut self.state.albums_list_state);
    }

    fn render_tracks(&mut self, frame: &mut Frame, area: Rect) {
        let style = if !self.tracks_selected {
            Color::LightBlue
        } else {
            Color::White
        };
        let block = Block::bordered().border_style(style).title("Tracks");
        let selected_album = self.state.albums_list_state.selected().unwrap();
        let tracks = self.library.get_tracks(selected_album);
        let items: Vec<&str> = tracks
            .iter()
            .map(|item| item.file_name().unwrap().to_str().unwrap())
            .collect();
        let list = List::new(items).highlight_symbol(">").block(block);

        frame.render_stateful_widget(list, area, &mut self.state.track_list_state);
    }
}
