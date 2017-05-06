extern crate xml;

use std::fs::File;
use std::io::BufReader;

use xml::reader::{EventReader, XmlEvent};

#[allow(dead_code)]
struct Track {
    title: String,
    artist_name: String,
    album_title: String,
    album_track_number: Option<u16>,
    duration_seconds: Option<f64>,
    bpm: Option<f64>,
}
impl Track {
    fn new_from_entry_elements(elements: &EntryElements) -> Track {
        Track {
            title: get_element_attribute(elements, "ENTRY", "TITLE").unwrap_or(String::new()),
            artist_name: get_element_attribute(elements, "ENTRY", "ARTIST")
                .unwrap_or(String::new()),
            album_title: get_element_attribute(elements, "ALBUM", "TITLE").unwrap_or(String::new()),
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
    let file = File::open("collection.nml").unwrap();
    let file = BufReader::new(file);

    let mut tracks: Vec<Track> = Vec::new();

    let parser = EventReader::new(file);
    println!("parsing collection.nml");
    for e in parser {
        let mut entry_elements = EntryElements::new();
        match e {
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
                            tracks.push(Track::new_from_entry_elements(&entry_elements));
                            entry_elements.clear();
                        }
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
    println!("done!");
}
