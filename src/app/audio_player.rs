use std::{fs::File, io::BufReader, path::PathBuf};

use rodio::{Decoder, MixerDeviceSink, Player};

pub struct AudioPlayer {
    #[allow(dead_code)]
    // sink must be preserved on entire App lifecycle to play audio
    sink: MixerDeviceSink,
    player: Player,
    current_track: Option<PathBuf>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let sink =
            rodio::DeviceSinkBuilder::open_default_sink().expect("can't open default audio stream");
        let player = Player::connect_new(sink.mixer());

        Self {
            sink,
            player,
            current_track: None,
        }
    }

    pub fn play(&self) {
        match self.current_track.as_ref() {
            Some(_) => {
                if self.player.is_paused() {
                    self.player.play();
                } else {
                    self.player.pause();
                }
            }
            None => {}
        }
    }

    pub fn set_current_track(&mut self, path: &PathBuf) {
        if !self.player.empty() {
            self.player.clear();
        }

        self.current_track = Some(path.clone());
        let file = BufReader::new(File::open(path.as_path()).unwrap());
        let source = Decoder::try_from(file).unwrap();
        self.player.append(source);
        self.player.play();
    }
}
