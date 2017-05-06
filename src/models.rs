use iron;
use juniper;
use std::sync::{Arc, Weak};
use std::cell::RefCell;

use parser::{Entry, get_element_attribute};
use utils::parse_option_str;

#[allow(dead_code)]
pub struct CollectionData {
    pub tracks: Vec<Arc<Track>>,
    pub artists: Vec<Arc<RefCell<Artist>>>,
    pub albums: Vec<Arc<RefCell<Album>>>,
}
impl CollectionData {
    pub fn new() -> CollectionData {
        CollectionData {
            tracks: Vec::new(),
            artists: Vec::new(),
            albums: Vec::new(),
        }
    }
    fn get_or_create_album_for_entry(&mut self, entry: &Entry) -> Option<Arc<RefCell<Album>>> {
        let title = get_element_attribute(&entry.elements, "ALBUM", "TITLE");
        match title {
            Some(title) => {
                match self.albums
                          .iter()
                          .find(|&x| x.borrow().title == title)
                          .map(|x| x.clone()) {
                    Some(album_ref) => Some(album_ref),
                    None => {
                        let album_ref = Arc::new(RefCell::new(Album::new(title)));
                        self.albums.push(album_ref.clone());
                        Some(album_ref)
                    }
                }
            }
            None => None,
        }
    }
    fn get_or_create_artist_for_entry(&mut self, entry: &Entry) -> Option<Arc<RefCell<Artist>>> {
        let name = get_element_attribute(&entry.elements, "ENTRY", "ARTIST");
        match name {
            Some(name) => {
                match self.artists
                          .iter()
                          .find(|&x| x.borrow().name == name)
                          .map(|x| x.clone()) {
                    Some(artist_ref) => Some(artist_ref),
                    None => {
                        let artist_ref = Arc::new(RefCell::new(Artist::new(name)));
                        self.artists.push(artist_ref.clone());
                        Some(artist_ref)
                    }
                }
            }
            None => None,
        }
    }
    pub fn add_entry(&mut self, entry: &Entry) {
        let album_option = self.get_or_create_album_for_entry(entry);
        let artist_option = self.get_or_create_artist_for_entry(entry);
        let track = Arc::new(Track::new(entry, artist_option.clone(), album_option.clone()));

        self.tracks.push(track.clone());

        if let Some(artist) = artist_option {
            let mut artist = artist.borrow_mut();
            artist.add_track(track.clone());
            if let Some(album) = album_option.clone() {
                artist.add_album(album.clone());
            }
        }

        if let Some(album) = album_option {
            album.borrow_mut().add_track(track.clone());
        }
    }
}
impl juniper::Context for CollectionData {}
// impl iron::typemap::Key for CollectionData {
//     type Value = CollectionData;
// }

#[allow(dead_code)]
pub struct Artist {
    pub name: String,
    albums: Vec<Weak<RefCell<Album>>>,
    tracks: Vec<Weak<Track>>,
}
impl Artist {
    fn new(name: String) -> Artist {
        Artist {
            name: name,
            albums: Vec::new(),
            tracks: Vec::new(),
        }
    }
    fn add_track(&mut self, track: Arc<Track>) {
        self.tracks.push(Arc::downgrade(&track));
    }
    fn add_album(&mut self, album: Arc<RefCell<Album>>) {
        let weak_ref = Arc::downgrade(&album);
        let contains_album = {
            self.albums
                .iter()
                .filter(|x| match x.upgrade() {
                            Some(filter_album) => *filter_album == *album,
                            None => false,
                        })
                .count() > 0
        };
        if !contains_album {
            self.albums.push(weak_ref);
        }
    }
}

#[allow(dead_code)]
pub struct Album {
    pub title: String,
    tracks: Vec<Weak<Track>>,
}
impl Album {
    fn new(title: String) -> Album {
        Album {
            title: title,
            tracks: Vec::new(),
        }
    }
    fn add_track(&mut self, track: Arc<Track>) {
        self.tracks.push(Arc::downgrade(&track));
    }
}
impl PartialEq for Album {
    fn eq(&self, other: &Album) -> bool {
        self.title == other.title
    }
}

#[allow(dead_code)]
pub struct Track {
    pub title: String,
    artist: Option<Weak<RefCell<Artist>>>,
    album: Option<Weak<RefCell<Album>>>,
    album_track_number: Option<u16>,
    duration_seconds: Option<f64>,
    bpm: Option<f64>,
}
impl Track {
    fn new(entry: &Entry,
           artist: Option<Arc<RefCell<Artist>>>,
           album: Option<Arc<RefCell<Album>>>)
           -> Track {
        let elements = &entry.elements;
        Track {
            title: get_element_attribute(elements, "ENTRY", "TITLE").unwrap_or(String::new()),
            artist: artist.map(|x| Arc::downgrade(&x)),
            album: album.map(|x| Arc::downgrade(&x)),
            album_track_number: get_element_attribute(elements, "ALBUM", "TRACK")
                .and_then(parse_option_str::<u16>),
            duration_seconds: get_element_attribute(elements, "INFO", "PLAYTIME_FLOAT")
                .and_then(parse_option_str::<f64>),
            bpm: get_element_attribute(elements, "INFO", "PLAYTIME_FLOAT")
                .and_then(parse_option_str::<f64>),
        }
    }
}
