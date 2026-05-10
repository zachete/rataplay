use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

pub struct Album {
    pub name: String,
    tracks: Vec<PathBuf>,
}

pub struct Library {
    albums: Vec<Album>,
}

impl Library {
    pub fn new() -> Self {
        Self { albums: Vec::new() }
    }

    pub fn load(&mut self) {
        let maybe_dir = fs::read_dir("./music");
        if let Ok(dir) = maybe_dir {
            for maybe_entry in dir {
                if let Ok(entry) = maybe_entry
                    && let Ok(metadata) = entry.metadata()
                {
                    if metadata.is_dir() {
                        let album = self.read_dir(entry);
                        self.albums.push(album);
                    }
                }
            }
        }
    }

    pub fn get_albums(&self) -> &Vec<Album> {
        &self.albums
    }

    pub fn get_tracks(&self, album_index: usize) -> &Vec<PathBuf> {
        &self
            .albums
            .get(album_index)
            .expect("album not found")
            .tracks
    }

    pub fn get_track(&self, album_index: usize, track_index: usize) -> &PathBuf {
        let current_album = self.albums.get(album_index).expect("album not found");
        let track = current_album
            .tracks
            .get(track_index)
            .expect("track not found");

        track
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
}
