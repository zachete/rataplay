use std::{
    borrow,
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

#[derive(Clone, Debug)]
pub struct Artist {
    pub name: String,
    // albums: Box<Vec<ModernAlbum>>,
}

#[derive(Clone, Debug)]
pub struct Song {
    pub title: String,
    pub path: PathBuf,
    // album_ref: Rc<RefCell<ModernAlbum>>,
}

#[derive(Debug)]
pub struct ModernAlbum {
    pub title: String,
    pub artist_ref: Rc<RefCell<Artist>>,
    pub songs: Vec<Song>,
}

pub struct ModernLibrary {
    pub artists: HashMap<String, Rc<RefCell<Artist>>>,
    pub albums: Vec<Rc<RefCell<ModernAlbum>>>,
    pub songs: Vec<Rc<RefCell<Song>>>,
}

impl Library {
    pub fn new() -> Self {
        let library = ModernLibrary {
            artists: HashMap::new(),
            albums: Vec::new(),
            songs: Vec::new(),
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

    pub fn get_artists(&self) -> &HashMap<String, Rc<RefCell<Artist>>> {
        &self.library.artists
    }

    pub fn get_albums(&self) -> &Vec<Rc<RefCell<ModernAlbum>>> {
        &self.library.albums
    }

    pub fn get_songs(&self, album_index: usize) -> Vec<Song> {
        let album_rc = self.library.albums.get(album_index).unwrap();
        let album_ref = album_rc.borrow();
        album_ref.songs.clone()
    }

    pub fn get_song(&self, artist: String, album_index: usize, song_index: usize) -> Song {
        let mut target_albums = self.library.albums.iter().filter(|item| {
            let album = item.borrow();
            let artist_ref = album.artist_ref.borrow();
            artist_ref.name == artist
        });

        tracing::info!("Album index: {}", album_index);
        tracing::info!("Target albums: {:?}", target_albums);
        let target_album = target_albums.nth(album_index).expect("album not found");
        let album_ref = target_album.borrow();
        album_ref
            .songs
            .get(song_index)
            .expect("song not found")
            .clone()
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
                        let track = tag.title().unwrap().to_string();

                        let artist_ref = if !self.library.artists.contains_key(&artist) {
                            let artist_ref = Rc::new(RefCell::new(Artist {
                                name: artist.clone(),
                            }));
                            self.library.artists.insert(artist, artist_ref.clone());
                            artist_ref.clone()
                        } else {
                            self.default_artist.clone()
                        };

                        match self
                            .library
                            .albums
                            .iter()
                            .position(|item| item.borrow().title == album)
                        {
                            Some(index) => {
                                self.library
                                    .albums
                                    .get(index)
                                    .unwrap()
                                    .borrow_mut()
                                    .songs
                                    .push(Song {
                                        title: track,
                                        path: entry.path(),
                                    });
                            }
                            None => {
                                self.library.albums.push(Rc::new(RefCell::new(ModernAlbum {
                                    title: album,
                                    artist_ref: artist_ref,
                                    songs: Vec::new(),
                                })));
                            }
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
