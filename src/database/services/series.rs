use super::super::models::data::{Part, Series};
use std::path::PathBuf;

pub struct SeriesService {
    // db
}

impl SeriesService {
    pub fn new() -> Self {
        Self {
            // db
        }
    }

    fn add_series(path: PathBuf, name: String) -> bool {
        false
    }

    fn get_series(lib_id: &i32) -> Option<Series> {
        None
    }

    fn get_all_series() -> Vec<Series> {
        vec![]
    }

    fn delete_series(lib_id: &i32) -> bool {
        false
    }

    fn get_all_parts_in_series(series_id: &i32) -> Vec<Part> {
        vec![]
    }
}
