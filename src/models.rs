use std::rc::{Rc, Weak};
use std::cell::RefCell;

use parser::{Node, NodeElements, get_element_attribute};
use utils::parse_option_str;

#[allow(dead_code)]
pub struct CollectionData {
    pub tracks: Vec<Rc<Track>>,
    pub artists: Vec<Rc<RefCell<Artist>>>,
    pub albums: Vec<Rc<RefCell<Album>>>,
}
impl CollectionData {
    pub fn new() -> CollectionData {
        CollectionData {
            tracks: Vec::new(),
            artists: Vec::new(),
            albums: Vec::new(),
        }
    }
    fn get_or_create_album_for_node(&mut self,
                                    node_elements: &NodeElements)
                                    -> Option<Rc<RefCell<Album>>> {
        let title = get_element_attribute(&node_elements, "ALBUM", "TITLE");
        match title {
            Some(title) => {
                match self.albums
                          .iter()
                          .find(|&x| x.borrow().title == title)
                          .map(|x| x.clone()) {
                    Some(album_ref) => Some(album_ref),
                    None => {
                        let album_ref = Rc::new(RefCell::new(Album::new(title)));
                        self.albums.push(album_ref.clone());
                        Some(album_ref)
                    }
                }
            }
            None => None,
        }
    }
    fn get_or_create_artist_for_node(&mut self,
                                     node_elements: &NodeElements)
                                     -> Option<Rc<RefCell<Artist>>> {
        let name = get_element_attribute(&node_elements, "ENTRY", "ARTIST");
        match name {
            Some(name) => {
                match self.artists
                          .iter()
                          .find(|&x| x.borrow().name == name)
                          .map(|x| x.clone()) {
                    Some(artist_ref) => Some(artist_ref),
                    None => {
                        let artist_ref = Rc::new(RefCell::new(Artist::new(name)));
                        self.artists.push(artist_ref.clone());
                        Some(artist_ref)
                    }
                }
            }
            None => None,
        }
    }
    fn add_track_node_elements(&mut self, node_elements: &NodeElements) {
        let album_option = self.get_or_create_album_for_node(node_elements);
        let artist_option = self.get_or_create_artist_for_node(node_elements);
        match Track::new(node_elements, &artist_option, &album_option) {
            Ok(track_inst) => {
                let track = Rc::new(track_inst);

                self.tracks.push(track.clone());
                if let Some(artist) = artist_option {
                    let mut artist = artist.borrow_mut();
                    artist.add_track(&track);
                    if let Some(ref album) = album_option {
                        artist.add_album(album);
                    }
                }
                if let Some(ref album) = album_option {
                    album.borrow_mut().add_track(&track);
                }
            }
            Err(_) => {}
        }
    }
    pub fn add_node(&mut self, node: &Node) {
        match *node {
            Node::Track { ref elements } => {
                self.add_track_node_elements(elements);
            }
            Node::Playlist { .. } => {}
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
    fn add_track(&mut self, track: &Rc<Track>) {
        self.tracks.push(Rc::downgrade(&track));
    }
    fn add_album(&mut self, album: &Rc<RefCell<Album>>) {
        let weak_ref = Rc::downgrade(album);
        let contains_album = {
            self.albums
                .iter()
                .filter(|x| match x.upgrade() {
                            Some(filter_album) => filter_album == *album,
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
    fn add_track(&mut self, track: &Rc<Track>) {
        self.tracks.push(Rc::downgrade(&track));
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
    fn new(elements: &NodeElements,
           artist: &Option<Rc<RefCell<Artist>>>,
           album: &Option<Rc<RefCell<Album>>>)
           -> Result<Track, &'static str> {
        let title = get_element_attribute(elements, "ENTRY", "TITLE");
        if title.is_none() {
            return Err("ENTRY does not have title");
        }

        Ok(Track {
               title: title.unwrap(),
               artist: match *artist {
                   Some(ref x) => Some(Rc::downgrade(x)),
                   None => None,
               },
               album: match *album {
                   Some(ref x) => Some(Rc::downgrade(x)),
                   None => None,
               },
               album_track_number: get_element_attribute(elements, "ALBUM", "TRACK")
                   .and_then(parse_option_str::<u16>),
               duration_seconds: get_element_attribute(elements, "INFO", "PLAYTIME_FLOAT")
                   .and_then(parse_option_str::<f64>),
               bpm: get_element_attribute(elements, "INFO", "PLAYTIME_FLOAT")
                   .and_then(parse_option_str::<f64>),
           })
    }
}
