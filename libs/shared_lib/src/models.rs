use std::{collections::HashMap};

// TODO: decide how I want to do efficient look up.
pub struct DataStoreServiceSchema {
    bucket: String,
    key: String,
    version: String,
    locations: Vec<ObjectLocation>,
    metadata: HashMap<String, String>,
}

// TODO: decide on structure in more details
pub struct ObjectLocation {
    service_name: String,
}