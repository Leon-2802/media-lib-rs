use super::super::models::data::Item;
use std::path::{Path, PathBuf};

pub struct LibService {
    // db
}

impl LibService {
    pub fn new() -> Self {
        Self {
            // db
        }
    }

    fn add_item(path: PathBuf, name: String) -> bool {
        false
    }

    fn get_item(lib_id: &i32) -> Option<Item> {
        None
    }

    fn get_all_items() -> Vec<Item> {
        vec![]
    }

    fn delete_item(lib_id: &i32) -> bool {
        false
    }
}
