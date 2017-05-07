extern crate xml;

mod models;
mod parser;
mod utils;

fn main() {
    let mut collection_data = models::CollectionData::new();
    let entries = parser::Entries::new("collection.nml");

    println!("parsing collection.nml");

    for entry in entries {
        collection_data.add_entry(&entry);
    }
    println!("done!");
    println!("tracks:  {}", collection_data.tracks.len());
    println!("artists: {}", collection_data.artists.len());
    println!("albums:  {}", collection_data.albums.len());

    for artist in collection_data.artists {
        println!("{}", artist.borrow().name);
        for track in artist.borrow().get_tracks() {
            println!("    {}", track.title);
        }
        println!("");
    }
}
