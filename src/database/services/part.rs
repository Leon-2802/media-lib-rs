use super::super::models::data::{Item, Part};
use std::path::PathBuf;

pub struct PartService {
    // db
}

impl PartService {
    pub fn new() -> Self {
        Self {
            // db
        }
    }

    fn add_part(path: PathBuf, name: String) -> bool {
        // add part to db
        false
    }

    fn get_part(part_id: &i32) -> Option<String> {
        // get part from db
        None
    }

    fn get_all_parts() -> Vec<Part> {
        // get all parts from db
        vec![]
    }

    fn delete_part(part_id: &i32) -> bool {
        // delete part from db
        false
    }

    fn get_all_items_in_part(part_id: &i32) -> Vec<Item> {
        vec![]
    }
}
