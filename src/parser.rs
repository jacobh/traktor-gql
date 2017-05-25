use std;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use xml;
use xml::reader::{EventReader, XmlEvent};

pub type NodeElements = Vec<XmlEvent>;
type NodeAttributes = Vec<xml::attribute::OwnedAttribute>;

pub enum NodeType {
    Track,
    Playlist,
}

pub struct Node {
    pub node_type: NodeType,
    pub attributes: NodeAttributes,
    pub elements: NodeElements,
}

enum RootNode {
    None,
    Collection,
    Playlists,
}

pub struct CollectionParser {
    _current_root_node: RootNode,
    _parser: EventReader<std::io::BufReader<File>>,
}

impl CollectionParser {
    pub fn new<P: AsRef<Path>>(collection_path: P) -> CollectionParser {
        let file = File::open(collection_path).unwrap();
        let file = BufReader::new(file);
        CollectionParser {
            _current_root_node: RootNode::None,
            _parser: EventReader::new(file),
        }
    }
}

impl Iterator for CollectionParser {
    type Item = Node;
    fn next(&mut self) -> Option<Node> {
        let mut node_elements = NodeElements::new();
        let mut node_attributes = NodeAttributes::new();
        loop {
            match self._parser.next() {
                Ok(e) => {
                    match e {
                        XmlEvent::StartElement { .. } => {
                            match self._current_root_node {
                                RootNode::None => {
                                    self._current_root_node = {
                                        match e {
                                            XmlEvent::StartElement { ref name, .. } => {
                                                match name.local_name.as_ref() {
                                                    "COLLECTION" => RootNode::Collection,
                                                    "PLAYLISTS" => RootNode::Playlists,
                                                    _ => RootNode::None,
                                                }
                                            }
                                            _ => RootNode::None,
                                        }
                                    };
                                    continue;
                                }
                                RootNode::Collection => {
                                    match node_attributes.is_empty() {
                                        true => {
                                            if let XmlEvent::StartElement { attributes, .. } = e {
                                                node_attributes = attributes;
                                            }
                                        }
                                        false => {
                                            node_elements.push(e);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        XmlEvent::EndElement { name } => {
                            match name.local_name.as_ref() {
                                "COLLECTION" | "PLAYLISTS" => {
                                    assert!(node_attributes.is_empty());
                                    assert!(node_elements.is_empty());
                                    self._current_root_node = RootNode::None;
                                }
                                "ENTRY" => {
                                    return Some(Node {
                                                    node_type: NodeType::Track,
                                                    attributes: node_attributes,
                                                    elements: node_elements,
                                                })
                                }
                                "NODE" => {
                                    return Some(Node {
                                                    node_type: NodeType::Playlist,
                                                    attributes: node_attributes,
                                                    elements: node_elements,
                                                })
                                }
                                _ => {}
                            }
                        }
                        XmlEvent::EndDocument => {
                            println!("end of doc");
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

fn get_element_with_name<'a, 'b>(elements: &'a NodeElements,
                                 lookup_name: &'b str)
                                 -> Option<&'a XmlEvent> {
    elements
        .iter()
        .find(|x| match *x {
                  &XmlEvent::StartElement { ref name, .. } => &name.local_name == lookup_name,
                  _ => false,
              })
}

pub fn get_attribute(attributes: &Vec<xml::attribute::OwnedAttribute>,
                     key: &str)
                     -> Option<String> {
    attributes
        .iter()
        .find(|&x| x.name.local_name == key)
        .and_then(|x| Some(x.value.clone()))
}

pub fn get_element_attribute(element: &XmlEvent, attribute_key: &str) -> Option<String> {
    match element {
        &XmlEvent::StartElement { ref attributes, .. } => get_attribute(&attributes, attribute_key),
        _ => None,
    }
}

pub fn get_elements_attribute(elements: &NodeElements,
                              element_name: &str,
                              attribute_key: &str)
                              -> Option<String> {
    match get_element_with_name(elements, element_name) {
        Some(element) => get_element_attribute(element, attribute_key),
        _ => None,
    }
}
