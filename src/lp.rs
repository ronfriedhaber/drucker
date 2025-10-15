use std::path::Path;

pub struct DruckerOptions {}

pub enum DruckerContent {
    Text(String),
    File(Path),
}
