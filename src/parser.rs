use std;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use xml;
use xml::reader::{EventReader, XmlEvent};

pub type NodeElements = Vec<XmlEvent>;

pub enum Node {
    Track { elements: NodeElements },
    Playlist { elements: NodeElements },
}

pub struct CollectionParser {
    _parser: EventReader<std::io::BufReader<File>>,
}

impl CollectionParser {
    pub fn new<P: AsRef<Path>>(collection_path: P) -> CollectionParser {
        let file = File::open(collection_path).unwrap();
        let file = BufReader::new(file);
        CollectionParser { _parser: EventReader::new(file) }
    }
}

impl Iterator for CollectionParser {
    type Item = Node;
    fn next(&mut self) -> Option<Node> {
        let mut node_elements = NodeElements::new();
        loop {
            match self._parser.next() {
                Ok(e) => {
                    match e {
                        XmlEvent::StartElement { .. } => {
                            match node_elements.is_empty() {
                                true => {
                                    let is_first_element_of_node = {
                                        match e {
                                            XmlEvent::StartElement { ref name, .. } => {
                                                name.local_name == "ENTRY" ||
                                                name.local_name == "NODE"
                                            }
                                            _ => false,
                                        }
                                    };
                                    if is_first_element_of_node {
                                        node_elements.push(e);
                                    }
                                }
                                false => {
                                    node_elements.push(e);
                                }
                            }
                        }
                        XmlEvent::EndElement { name } => {
                            match name.local_name.as_ref() {
                                "ENTRY" => return Some(Node::Track { elements: node_elements }),
                                "NODE" => return Some(Node::Playlist { elements: node_elements }),
                                _ => {}
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

fn get_attribute(attributes: &Vec<xml::attribute::OwnedAttribute>, key: &str) -> Option<String> {
    attributes
        .iter()
        .find(|&x| x.name.local_name == key)
        .and_then(|x| Some(x.value.clone()))
}

pub fn get_element_attribute(elements: &NodeElements,
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
