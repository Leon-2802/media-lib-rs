use super::super::models::data::{Library, Series};
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

    fn add_library(path: PathBuf, name: String) -> bool {
        false
    }

    fn get_library(lib_id: &i32) -> Option<Library> {
        None
    }

    fn get_all_libraries() -> Vec<Library> {
        vec![]
    }

    fn delete_library(lib_id: &i32) -> bool {
        false
    }

    fn get_series_in_library(lib_id: &i32) -> Vec<Series> {
        vec![]
    }
}
