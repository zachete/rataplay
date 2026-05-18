use crate::app::{
    audio_player::AudioPlayer, library::Artist, library::Library, library::ModernAlbum,
    library::Song,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::{Block, List, ListState},
};

#[derive(PartialEq)]
enum LayoutState {
    ARTISTS,
    ALBUMS,
    SONGS,
}

struct State {
    layout: LayoutState,
    selected_track: Option<Song>,
    selected_artist: Option<String>,
    artists_list: Vec<Artist>,
    albums_list: Vec<ModernAlbum>,
    songs_list: Vec<Song>,
    artists_list_state: ListState,
    albums_list_state: ListState,
    songs_list_state: ListState,
}

pub struct App {
    audio_player: AudioPlayer,
    state: State,
    library: Library,
    prev_selected: Option<usize>,
}

impl App {
    pub fn new() -> Self {
        let audio_player = AudioPlayer::new();
        let state = State {
            layout: LayoutState::ARTISTS,
            selected_track: None,
            selected_artist: None,
            artists_list_state: ListState::default().with_selected(Some(0)),
            albums_list_state: ListState::default().with_selected(Some(0)),
            songs_list_state: ListState::default().with_selected(Some(0)),
            artists_list: Vec::new(),
            songs_list: Vec::new(),
            albums_list: Vec::new(),
        };
        let mut library = Library::new();
        library.load();

        let app = Self {
            audio_player,
            state,
            library,
            prev_selected: None,
        };

        app
    }

    pub fn previous(&mut self) {
        match self.state.layout {
            LayoutState::ARTISTS => {
                self.state.artists_list_state.select_previous();
                self.state.songs_list_state.select_first();
            }
            LayoutState::ALBUMS => {
                self.state.albums_list_state.select_previous();
            }
            LayoutState::SONGS => {
                self.state.songs_list_state.select_previous();
            }
        }

        self.set_selected_track();
    }

    pub fn next(&mut self) {
        match self.state.layout {
            LayoutState::ARTISTS => {
                self.state.artists_list_state.select_next();
                self.state.albums_list_state.select_first();
                self.state.songs_list_state.select_first();
            }
            LayoutState::ALBUMS => {
                self.state.albums_list_state.select_next();
                self.state.songs_list_state.select_first();
            }
            LayoutState::SONGS => {
                let selected_song_index = self.state.songs_list_state.selected().unwrap();
                if selected_song_index < self.state.songs_list.len() - 1 {
                    self.state.songs_list_state.select_next();
                }
            }
        }

        self.set_selected_track();
    }

    fn set_selected_track(&mut self) {
        let artist_index = self.state.albums_list_state.selected().unwrap();
        let artist = self.state.artists_list.get(artist_index).unwrap();
        let album_index = self.state.albums_list_state.selected().unwrap();
        let song_index = self.state.songs_list_state.selected().unwrap();
        let song = self
            .library
            .get_song(artist.name.clone(), album_index, song_index);
        self.state.selected_track = Some(song);
    }

    pub fn switch_layout_to_right(&mut self) {
        self.state.layout = match self.state.layout {
            LayoutState::ARTISTS => LayoutState::ALBUMS,
            LayoutState::ALBUMS => LayoutState::SONGS,
            LayoutState::SONGS => LayoutState::SONGS,
        }
    }

    pub fn switch_layout_to_left(&mut self) {
        self.state.layout = match self.state.layout {
            LayoutState::ARTISTS => LayoutState::ARTISTS,
            LayoutState::ALBUMS => LayoutState::ARTISTS,
            LayoutState::SONGS => LayoutState::ALBUMS,
        }
    }

    pub fn play(&mut self) {
        let selected = self.state.songs_list_state.selected().unwrap();

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
        let song = self.state.selected_track.as_ref().unwrap();
        self.audio_player.set_current_track(&song.path);
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::horizontal([
            Constraint::Length(30),
            Constraint::Length(30),
            Constraint::Length(30),
        ])
        .spacing(1);

        let [artists, albums, songs] = frame.area().layout(&layout);
        self.render_artists(frame, artists);
        self.render_albums(frame, albums);
        self.render_songs(frame, songs);
    }

    pub fn render_artists(&mut self, frame: &mut Frame, area: Rect) {
        let style = if self.state.layout == LayoutState::ARTISTS {
            Color::White
        } else {
            Color::LightBlue
        };
        let block = Block::bordered().border_style(style).title("Artists");
        self.state.artists_list = self
            .library
            .get_artists()
            .iter()
            .map(|(_, item)| item.borrow().clone())
            .collect();
        let items: Vec<String> = self
            .state
            .artists_list
            .iter()
            .map(|item| item.name.clone())
            .collect();
        let list = List::new(items).highlight_symbol(">").block(block);
        self.state.selected_artist = Some(self.state.artists_list[0].name.clone());
        frame.render_stateful_widget(list, area, &mut self.state.artists_list_state);
    }

    pub fn render_albums(&mut self, frame: &mut Frame, area: Rect) {
        let style = if self.state.layout == LayoutState::ALBUMS {
            Color::White
        } else {
            Color::LightBlue
        };
        let block = Block::bordered().border_style(style).title("Albums");
        let items: Vec<String> = self
            .library
            .get_albums()
            .iter()
            .filter(|item| {
                let album_ref = item.borrow();
                let artist_ref = album_ref.artist_ref.borrow();
                artist_ref.name == self.state.selected_artist.clone().unwrap()
            })
            .into_iter()
            .map(|item| item.borrow().title.clone())
            .collect();
        let list = List::new(items).highlight_symbol(">").block(block);

        frame.render_stateful_widget(list, area, &mut self.state.albums_list_state);
    }

    fn render_songs(&mut self, frame: &mut Frame, area: Rect) {
        let style = if self.state.layout == LayoutState::SONGS {
            Color::White
        } else {
            Color::LightBlue
        };
        let block = Block::bordered().border_style(style).title("Songs");
        let selected_album = self.state.albums_list_state.selected().unwrap();
        self.state.songs_list = self.library.get_songs(selected_album);
        let items: Vec<&str> = self
            .state
            .songs_list
            .iter()
            .map(|item| item.title.as_str())
            .collect();
        tracing::info!("items: {:?}", items);
        let list = List::new(items).highlight_symbol(">").block(block);

        frame.render_stateful_widget(list, area, &mut self.state.songs_list_state);
    }
}
