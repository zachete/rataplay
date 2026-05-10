use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::{Block, List, ListState},
};
use rodio::{Decoder, MixerDeviceSink, Player};
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

pub struct App {
    state: ListState,
    tracks: Vec<PathBuf>,
    #[allow(dead_code)]
    // sink must be preserved on entire app lifecycle
    sink: MixerDeviceSink,
    prev_selected: Option<usize>,
    player: Player,
}

impl App {
    pub fn new() -> Self {
        let sink =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        let player = Player::connect_new(sink.mixer());

        let mut app = Self {
            state: ListState::default().with_selected(Some(0)),
            tracks: Vec::new(),
            sink,
            player,
            prev_selected: None,
        };

        app.read_music();

        app
    }

    pub fn previous(&mut self) {
        self.state.select_previous();
    }

    pub fn next(&mut self) {
        self.state.select_next();
    }

    pub fn read_music(&mut self) {
        let dir = fs::read_dir("./music");
        if dir.is_ok() {
            for entry in dir.unwrap() {
                if entry.is_ok() {
                    let file = entry.unwrap();

                    self.tracks.push(file.path())
                }
            }
        }
    }

    pub fn play(&mut self) {
        let selected = self.state.selected().unwrap();

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

        let file_path_buf = self.tracks.get(index).expect("track not found");
        let file = BufReader::new(File::open(file_path_buf.as_path()).unwrap());
        let source = Decoder::try_from(file).unwrap();
        self.player.append(source);
        self.player.play();
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let layout =
            Layout::horizontal([Constraint::Length(30), Constraint::Length(30)]).spacing(1);

        let [dirs, files] = frame.area().layout(&layout);
        self.render_dirs(frame, dirs);
        self.render_files(frame, files);
    }

    pub fn render_dirs(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().title("Albums");

        let items: Vec<&str> = self
            .tracks
            .iter()
            .map(|item| item.file_name().unwrap().to_str().unwrap())
            .collect();
        let list = List::new(items)
            .style(Color::LightBlue)
            .highlight_symbol(">")
            .block(block);

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn render_files(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered().title("Tracks");

        frame.render_widget(block, area);
    }
}
