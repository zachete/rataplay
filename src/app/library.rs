use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{self, DirEntry},
    path::PathBuf,
    rc::Rc,
};

use lofty::{file::TaggedFileExt, read_from_path, tag::Accessor};

pub struct Album {
    pub name: String,
    tracks: Vec<PathBuf>,
}

pub struct Library {
    albums: Vec<Album>,
    library: ModernLibrary,
    default_artist: Rc<RefCell<Artist>>,
}

pub struct Artist {
    name: String,
    // albums: Box<Vec<ModernAlbum>>,
}

pub struct Song {
    title: String,
    path: PathBuf,
}

pub struct ModernAlbum {
    pub title: String,
    artist_ref: Rc<RefCell<Artist>>,
    // songs: Vec<Song>
}

pub struct ModernLibrary {
    pub artists: HashMap<String, Rc<RefCell<Artist>>>,
    pub albums: Vec<Rc<RefCell<ModernAlbum>>>,
}

impl Library {
    pub fn new() -> Self {
        let library = ModernLibrary {
            artists: HashMap::new(),
            albums: Vec::new(),
        };
        let default_artist = Rc::new(RefCell::new(Artist {
            name: String::from("Other"),
        }));

        Self {
            albums: Vec::new(),
            library,
            default_artist,
        }
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

    pub fn get_albums(&self) -> &Vec<Rc<RefCell<ModernAlbum>>> {
        &self.library.albums
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
                let is_audio = match entry.path().extension() {
                    Some(val) => val == "mp3",
                    None => false,
                };

                if metadata.is_file() && is_audio {
                    let tagged_track =
                        read_from_path(entry.path()).expect("can't get tagget audio file");

                    if let Some(tag) = tagged_track.primary_tag() {
                        let artist = tag.artist().unwrap().to_string();
                        let album = tag.album().unwrap().to_string();
                        let track = tag.track().unwrap().to_string();

                        let artist_ref = if !self.library.artists.contains_key(&artist) {
                            let artist_ref = Rc::new(RefCell::new(Artist {
                                name: artist.clone(),
                            }));
                            self.library.artists.insert(artist, artist_ref.clone());
                            artist_ref.clone()
                        } else {
                            self.default_artist.clone()
                        };

                        if self
                            .library
                            .albums
                            .iter()
                            .find(|item| item.borrow().title == album)
                            .is_none()
                        {
                            self.library.albums.push(Rc::new(RefCell::new(ModernAlbum {
                                title: album,
                                artist_ref: artist_ref,
                            })));
                        }
                    }

                    album.tracks.push(entry.path());
                } else if metadata.is_dir() {
                }
            }
        }

        album
    }
}
