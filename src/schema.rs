use models::{CollectionData, Track, Album, Artist};

graphql_object!(Track: CollectionData |&self| {
    field title() -> &String {
        &self.title
    }
});

graphql_object!(Album: CollectionData |&self| {
    field title() -> &String {
        &self.title
    }
});

graphql_object!(Artist: CollectionData |&self| {
    field name() -> &String {
        &self.name
    }
});


pub struct QueryRoot;
graphql_object!(QueryRoot: CollectionData |&self| {
    description: "Root query"
});
