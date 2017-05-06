extern crate iron;
extern crate mount;
extern crate persistent;
#[macro_use]
extern crate juniper;
extern crate xml;

mod models;
mod parser;
mod schema;
mod utils;

use std::sync::RwLock;
use iron::prelude::*;
use mount::Mount;
use juniper::EmptyMutation;
use juniper::iron_handlers::{GraphQLHandler, GraphiQLHandler};
use persistent::Read;

use models::CollectionData;

fn context_factory(_: &mut Request) -> CollectionData {
    // req.get::<Read<models::CollectionData>>().unwrap()
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

    collection_data
}

fn main() {
    // Set up the web server

    let mut mount = Mount::new();

    let graphql_handler = GraphQLHandler::new(context_factory,
                                              schema::QueryRoot,
                                              EmptyMutation::<CollectionData>::new());
    let graphiql_handler = GraphiQLHandler::new("/graphql");

    mount.mount("/graphql", graphql_handler);
    mount.mount("/graphiql", graphiql_handler);

    let mut chain = Chain::new(mount);

    // // set up postgres connection pool
    // chain.link(Read::<CollectionData>::both(collection_data));

    let port = utils::get_env_var("PORT").unwrap_or("4000".to_string());
    Iron::new(chain)
        .http(format!("0.0.0.0:{}", port))
        .unwrap();
}
