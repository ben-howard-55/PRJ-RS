use std::collections::HashMap;

use bytes::Bytes;

// TODO: decide how I want to do efficient look up.
#[derive(Clone)]
pub struct DataStoreServiceSchema {
    bucket: String,
    key: String,
    version: String,
    locations: Vec<ObjectServer>,
    metadata: HashMap<String, String>,
}

// TODO: decide on structure in more details???
#[derive(Clone)]
pub struct ObjectServer {
    service_name: String,
}

#[derive(Clone)]
pub struct ObjectLocation {
    bucket: String,
    key: String,
    version: String,
    parts: Vec<ObjectPartLocation>,
}


#[derive(Clone)]
pub struct ObjectPartLocation {
    file_location: String,
    byte_offset: u32,
    length: u32,
    check_sum: u32
}

#[derive(Clone, Debug)]
pub struct CreateMultipartUploadRequest {
    bucket: String,
    key: String,
    version: String,
}

pub struct UploadPart {
    upload_id: String,
    bytes: Bytes
}

pub struct CloseMultipartUploadRequest {
    upload_id: String,
    part_order: Vec<String>
}

pub struct UploadObject {
    bucket: String,
    key: String,
    version: String,
    bytes: Bytes,
}