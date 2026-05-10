use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::{Block, List, ListState},
};
use rodio::{Decoder, MixerDeviceSink, Player};
use std::{
    fs::{self, DirEntry, File},
    io::BufReader,
    path::PathBuf,
};

struct Album {
    name: String,
    tracks: Vec<PathBuf>,
}

struct State {
    current_track: Option<PathBuf>,
    albums_list_state: ListState,
    track_list_state: ListState,
}

pub struct App {
    state: State,
    #[allow(dead_code)]
    // sink must be preserved on entire App lifecycle to play audio
    sink: MixerDeviceSink,
    player: Player,
    prev_selected: Option<usize>,
    library: Vec<Album>,
    tracks_selected: bool,
}

impl App {
    pub fn new() -> Self {
        let sink =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        let player = Player::connect_new(sink.mixer());
        let state = State {
            current_track: None,
            albums_list_state: ListState::default().with_selected(Some(0)),
            track_list_state: ListState::default().with_selected(Some(0)),
        };

        let mut app = Self {
            state,
            sink,
            player,
            library: Vec::new(),
            prev_selected: None,
            tracks_selected: false,
        };

        app.read_music();

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
        let current_album = self
            .library
            .get(self.state.albums_list_state.selected().unwrap())
            .expect("album not found");
        let track = current_album
            .tracks
            .get(self.state.track_list_state.selected().unwrap())
            .expect("track not found");
        self.state.current_track = Some(track.clone());
    }

    pub fn focus_albums(&mut self) {
        self.tracks_selected = false
    }

    pub fn focus_tracks(&mut self) {
        self.tracks_selected = true
    }

    pub fn read_music(&mut self) {
        let maybe_dir = fs::read_dir("./music");
        if let Ok(dir) = maybe_dir {
            for maybe_entry in dir {
                if let Ok(entry) = maybe_entry
                    && let Ok(metadata) = entry.metadata()
                {
                    if metadata.is_dir() {
                        let album = self.read_dir(entry);
                        self.library.push(album);
                    }
                }
            }
        }
    }

    fn read_dir(&mut self, dir_entry: DirEntry) -> Album {
        let dir = fs::read_dir(dir_entry.path()).unwrap();
        let dir_path = dir_entry.path();

        let mut album = Album {
            name: dir_path.file_name().unwrap().to_str().unwrap().to_string(),
            tracks: Vec::new(),
        };

        for maybe_entry in dir {
            if let Ok(entry) = maybe_entry
                && let Ok(metadata) = entry.metadata()
            {
                if metadata.is_file() {
                    album.tracks.push(entry.path());
                }
            }
        }

        album
    }

    pub fn play(&mut self) {
        let selected = self.state.track_list_state.selected().unwrap();

        match self.prev_selected {
            Some(val) => {
                if selected != val {
                    self.play_selected_file(selected);
                } else {
                    if self.player.is_paused() {
                        self.player.play();
                    } else {
                        self.player.pause();
                    }
                }
            }
            None => {
                self.play_selected_file(selected);
            }
        }

        self.prev_selected = Some(selected);
    }

    fn play_selected_file(&mut self, index: usize) {
        if !self.player.empty() {
            self.player.clear();
        }

        tracing::info!("play track: {}", index);

        let file_path_buf = self.state.current_track.as_ref().unwrap();
        let file = BufReader::new(File::open(file_path_buf.as_path()).unwrap());
        let source = Decoder::try_from(file).unwrap();
        self.player.append(source);
        self.player.play();
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
        let items: Vec<String> = self.library.iter().map(|item| item.name.clone()).collect();
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
        let selected = self.state.albums_list_state.selected().unwrap();
        let album = self
            .library
            .get(selected)
            .expect("selected album not found");
        let items: Vec<&str> = album
            .tracks
            .iter()
            .map(|item| item.file_name().unwrap().to_str().unwrap())
            .collect();
        let list = List::new(items).highlight_symbol(">").block(block);

        frame.render_stateful_widget(list, area, &mut self.state.track_list_state);
    }
}
