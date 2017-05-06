extern crate xml;

use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::rc::{Rc, Weak};

use xml::reader::{EventReader, XmlEvent};

#[allow(dead_code)]
struct CollectionData {
    tracks: Vec<Rc<Track>>,
    artists: Vec<Rc<Artist>>,
    albums: Vec<Rc<Album>>,
}
impl CollectionData {
    fn new() -> CollectionData {
        CollectionData {
            tracks: Vec::new(),
            artists: Vec::new(),
            albums: Vec::new(),
        }
    }
    fn get_or_create_album_for_entry(&mut self, entry: &Entry) -> Option<Rc<Album>> {
        let title = get_element_attribute(&entry.elements, "ALBUM", "TITLE");
        match title {
            Some(title) => {
                match self.albums
                          .iter()
                          .find(|&x| x.title == title)
                          .map(|x| x.clone()) {
                    Some(album_ref) => Some(album_ref),
                    None => {
                        let album_ref = Rc::new(Album::new(title));
                        self.albums.push(album_ref.clone());
                        Some(album_ref)
                    }
                }
            }
            None => None,
        }
    }
    fn get_or_create_artist_for_entry(&mut self, entry: &Entry) -> Option<Rc<Artist>> {
        let name = get_element_attribute(&entry.elements, "ENTRY", "ARTIST");
        match name {
            Some(name) => {
                match self.artists
                          .iter()
                          .find(|&x| x.name == name)
                          .map(|x| x.clone()) {
                    Some(artist_ref) => Some(artist_ref),
                    None => {
                        let artist_ref = Rc::new(Artist::new(name));
                        self.artists.push(artist_ref.clone());
                        Some(artist_ref)
                    }
                }
            }
            None => None,
        }
    }
    fn add_entry(&mut self, entry: &Entry) {
        let mut album_option = self.get_or_create_album_for_entry(entry);
        let mut artist_option = self.get_or_create_artist_for_entry(entry);
        let track_ref = Rc::new(Track::new(entry, artist_option, album_option.clone()));

        self.tracks.push(track_ref.clone());

        // if let Some(mut album) = album_option {
        //     album.add_track(track_ref.clone());
        // }
        // if let Some(mut artist) = artist_option {
        //     artist.add_track(track_ref.clone());
        //     if let Some(album) = album_option {
        //         artist.add_album(album.clone());
        //     }
        // }
    }
}

#[allow(dead_code)]
struct Artist {
    name: String,
    albums: Vec<Weak<Album>>,
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
    fn add_album(&mut self, album: Rc<Album>) {
        let weak_ref = Rc::downgrade(&album);
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
struct Album {
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
struct Track {
    title: String,
    artist: Option<Weak<Artist>>,
    album: Option<Weak<Album>>,
    album_track_number: Option<u16>,
    duration_seconds: Option<f64>,
    bpm: Option<f64>,
}
impl Track {
    fn new(entry: &Entry, artist: Option<Rc<Artist>>, album: Option<Rc<Album>>) -> Track {
        let elements = &entry.elements;
        Track {
            title: get_element_attribute(elements, "ENTRY", "TITLE").unwrap_or(String::new()),
            artist: artist.map(|x| Rc::downgrade(&x)),
            album: album.map(|x| Rc::downgrade(&x)),
            album_track_number: get_element_attribute(elements, "ALBUM", "TRACK")
                .and_then(parse_option_str::<u16>),
            duration_seconds: get_element_attribute(elements, "INFO", "PLAYTIME_FLOAT")
                .and_then(parse_option_str::<f64>),
            bpm: get_element_attribute(elements, "INFO", "PLAYTIME_FLOAT")
                .and_then(parse_option_str::<f64>),
        }
    }
}

type EntryElements = Vec<XmlEvent>;

struct Entry {
    elements: EntryElements,
}

struct Entries {
    _parser: EventReader<std::io::BufReader<File>>,
}

impl Entries {
    fn new<P: AsRef<Path>>(collection_path: P) -> Entries {
        let file = File::open(collection_path).unwrap();
        let file = BufReader::new(file);
        Entries { _parser: EventReader::new(file) }
    }
}

impl Iterator for Entries {
    type Item = Entry;
    fn next(&mut self) -> Option<Entry> {
        let mut entry_elements = EntryElements::new();
        loop {
            match self._parser.next() {
                Ok(e) => {
                    match e {
                        XmlEvent::StartElement { .. } => {
                            match entry_elements.is_empty() {
                                true => {
                                    let is_entry = {
                                        match e {
                                            XmlEvent::StartElement { ref name, .. } => {
                                                name.local_name == "ENTRY"
                                            }
                                            _ => false,
                                        }
                                    };
                                    if is_entry {
                                        entry_elements.push(e);
                                    }
                                }
                                false => {
                                    entry_elements.push(e);
                                }
                            }
                        }
                        XmlEvent::EndElement { name } => {
                            if name.local_name == "ENTRY" {
                                return Some(Entry { elements: entry_elements });
                            }
                        }
                        XmlEvent::EndDocument => {
                            break;
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            }
        }
        return None;
    }
}

fn get_element_with_name<'a, 'b>(elements: &'a EntryElements,
                                 lookup_name: &'b str)
                                 -> Option<&'a XmlEvent> {
    elements
        .iter()
        .find(|x| match *x {
                  &XmlEvent::StartElement { ref name, .. } => &name.local_name == lookup_name,
                  _ => false,
              })
}

fn get_attribute(attributes: &Vec<xml::attribute::OwnedAttribute>, key: &str) -> Option<String> {
    attributes
        .iter()
        .find(|&x| x.name.local_name == key)
        .and_then(|x| Some(x.value.clone()))
}

fn get_element_attribute(elements: &EntryElements,
                         element_name: &str,
                         attribute_key: &str)
                         -> Option<String> {
    match get_element_with_name(elements, element_name) {
        Some(element) => {
            match element {
                &XmlEvent::StartElement { ref attributes, .. } => {
                    get_attribute(&attributes, attribute_key)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn parse_option_str<T>(x: String) -> Option<T>
    where T: std::str::FromStr
{
    match x.parse::<T>() {
        Ok(x) => Some(x),
        Err(_) => None,
    }
}

fn main() {
    let mut collection_data = CollectionData::new();
    let entries = Entries::new("collection.nml");

    println!("parsing collection.nml");

    for entry in entries {
        collection_data.add_entry(&entry);
    }
    println!("done!");
    println!("tracks:  {}", collection_data.tracks.len());
    println!("artists: {}", collection_data.artists.len());
    println!("albums:  {}", collection_data.albums.len());
}
