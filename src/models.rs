use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;

use parser::{Node, NodeType, get_element_with_name, get_attribute, get_element_attribute,
             get_elements_attribute};

#[allow(dead_code)]
pub struct CollectionData {
    pub tracks: Vec<Rc<Track>>,
    pub artists: Vec<Rc<RefCell<Artist>>>,
    pub albums: Vec<Rc<RefCell<Album>>>,
    pub playlists: Vec<Rc<RefCell<Playlist>>>,
}
impl CollectionData {
    pub fn new() -> CollectionData {
        CollectionData {
            tracks: Vec::new(),
            artists: Vec::new(),
            albums: Vec::new(),
            playlists: Vec::new(),
        }
    }
    fn get_track_map(&self) -> HashMap<String, Rc<Track>> {
        self.tracks
            .iter()
            .map(|track| (track.location.as_primary_key(), track.clone()))
            .collect()
    }
    fn get_or_create_album_for_node(&mut self, node: &Node) -> Option<Rc<RefCell<Album>>> {
        let title = get_elements_attribute(&node.elements, "ALBUM", "TITLE");
        match title {
            Some(title) => {
                match self.albums
                          .iter()
                          .find(|&x| x.borrow().title == title)
                          .map(|x| x.clone()) {
                    Some(album) => Some(album),
                    None => {
                        let album = Rc::new(RefCell::new(Album::new(title)));
                        self.albums.push(album.clone());
                        Some(album)
                    }
                }
            }
            None => None,
        }
    }
    fn get_or_create_artist_for_node(&mut self, node: &Node) -> Option<Rc<RefCell<Artist>>> {
        let name = get_attribute(&node.attributes, "ARTIST");
        match name {
            Some(name) => {
                match self.artists
                          .iter()
                          .find(|&x| x.borrow().name == name)
                          .map(|x| x.clone()) {
                    Some(artist) => Some(artist),
                    None => {
                        let artist = Rc::new(RefCell::new(Artist::new(name)));
                        self.artists.push(artist.clone());
                        Some(artist)
                    }
                }
            }
            None => None,
        }
    }
    fn add_track_node(&mut self, node: &Node) {
        let album = self.get_or_create_album_for_node(node);
        let artist = self.get_or_create_artist_for_node(node);
        match Track::new(node, artist.clone(), album.clone()) {
            Ok(track) => {
                let track = Rc::new(track);

                self.tracks.push(track.clone());
                if let Some(artist) = artist {
                    let mut artist = artist.borrow_mut();
                    artist.add_track(track.clone());
                    if let Some(album) = album.clone() {
                        artist.add_album(album);
                    }
                }
                if let Some(album) = album {
                    album.borrow_mut().add_track(track);
                }
            }
            Err(_) => {}
        }
    }
    fn add_playlist_node(&mut self, node: &Node) {
        let tracks: Vec<Weak<Track>> = {
            let track_map = self.get_track_map();
            node.elements
                .iter()
                .filter_map(|elem| get_element_attribute(elem, "KEY"))
                .filter_map(|key| track_map.get(&key))
                .map(|track| Rc::downgrade(track))
                .collect()
        };

        println!("tracks in playlist: {}", tracks.len());

        if let Some(name) = get_attribute(&node.attributes, "NAME") {
            self.playlists
                .push(Rc::new(RefCell::new(Playlist::new(name, tracks))));
        }
    }
    pub fn add_node(&mut self, node: &Node) {
        match node.node_type {
            NodeType::Track => {
                self.add_track_node(&node);
            }
            NodeType::Playlist => {
                self.add_playlist_node(&node);
            }
        }
    }
}

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
    fn add_track(&mut self, track: Rc<Track>) {
        self.tracks.push(Rc::downgrade(&track));
    }
    fn add_album(&mut self, album: Rc<RefCell<Album>>) {
        let weak_ref = Rc::downgrade(&album);
        let contains_album = {
            self.albums
                .iter()
                .filter(|x| match x.upgrade() {
                            Some(filter_album) => filter_album == album,
                            None => false,
                        })
                .count() > 0
        };
        if !contains_album {
            self.albums.push(weak_ref);
        }
    }
    pub fn get_tracks(&self) -> Vec<Rc<Track>> {
        self.tracks
            .iter()
            .filter_map(|x| x.upgrade())
            .collect()
    }
}

#[allow(dead_code)]
pub struct Album {
    title: String,
    tracks: Vec<Weak<Track>>,
}
impl Album {
    fn new(title: String) -> Album {
        Album {
            title: title,
            tracks: Vec::new(),
        }
    }
    fn add_track(&mut self, track: Rc<Track>) {
        self.tracks.push(Rc::downgrade(&track));
    }
}
impl PartialEq for Album {
    fn eq(&self, other: &Album) -> bool {
        self.title == other.title
    }
}

#[allow(dead_code)]
struct TrackLocation {
    volume: String,
    path: String,
    filename: String,
}
impl TrackLocation {
    fn new_from_node(node: &Node) -> TrackLocation {
        let elem = get_element_with_name(&node.elements, "LOCATION").expect("All tracks must have LOCATION element");
        TrackLocation {
            volume: get_element_attribute(elem, "VOLUME").expect("VOLUME should always be set"),
            path: get_element_attribute(elem, "DIR").expect("DIR should always be set"),
            filename: get_element_attribute(elem, "FILE").expect("FILE should always be set"),
        }
    }
    fn as_primary_key(&self) -> String {
        format!("{}{}{}", self.volume, self.path, self.filename)
    }
}

#[allow(dead_code)]
pub struct Track {
    pub title: String,
    location: TrackLocation,
    artist: Option<Weak<RefCell<Artist>>>,
    album: Option<Weak<RefCell<Album>>>,
    album_track_number: Option<u16>,
    duration_seconds: Option<f64>,
    bpm: Option<f64>,
}
impl Track {
    fn new(node: &Node,
           artist: Option<Rc<RefCell<Artist>>>,
           album: Option<Rc<RefCell<Album>>>)
           -> Result<Track, &'static str> {
        let title = get_attribute(&node.attributes, "TITLE");
        if title.is_none() {
            return Err("ENTRY does not have title");
        }

        Ok(Track {
               title: title.unwrap(),
               location: TrackLocation::new_from_node(node),
               artist: artist.as_ref().map(|x| Rc::downgrade(x)),
               album: album.as_ref().map(|x| Rc::downgrade(x)),
               album_track_number: get_elements_attribute(&node.elements, "ALBUM", "TRACK")
                   .and_then(|x| x.parse().ok()),
               duration_seconds: get_elements_attribute(&node.elements, "INFO", "PLAYTIME_FLOAT")
                   .and_then(|x| x.parse().ok()),
               bpm: get_elements_attribute(&node.elements, "INFO", "PLAYTIME_FLOAT")
                   .and_then(|x| x.parse().ok()),
           })
    }
}

#[allow(dead_code)]
pub struct Playlist {
    pub name: String,
    tracks: Vec<Weak<Track>>,
}
impl Playlist {
    fn new(name: String, tracks: Vec<Weak<Track>>) -> Playlist {
        Playlist {
            name: name,
            tracks: tracks,
        }
    }
}
