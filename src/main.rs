extern crate xml;

mod models;
mod parser;
mod utils;

fn main() {
    let mut collection_data = models::CollectionData::new();
    let collection_parser = parser::CollectionParser::new("collection.nml");

    println!("parsing collection.nml");

    for node in collection_parser {
        collection_data.add_node(&node);
    }
    println!("done!");
    println!("tracks:  {}", collection_data.tracks.len());
    println!("artists: {}", collection_data.artists.len());
    println!("albums:  {}", collection_data.albums.len());

    for artist in collection_data.artists.iter().take(5) {
        println!("{}", artist.borrow().name);
        for track in artist.borrow().get_tracks() {
            println!("    {}", track.title);
        }
        println!("");
    }
}
