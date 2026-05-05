use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

use crate::database::models::data::{Entry, Library, LibraryKind};

pub struct LibraryService {
    conn: Arc<Mutex<Connection>>,
}

impl LibraryService {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn add(&self, _name: &str, _path: &Path, _kind: LibraryKind) -> Result<i64> {
        todo!()
    }

    pub fn get(&self, _id: i64) -> Result<Option<Library>> {
        todo!()
    }

    pub fn all(&self) -> Result<Vec<Library>> {
        todo!()
    }

    pub fn delete(&self, _id: i64) -> Result<bool> {
        todo!()
    }

    /// Top-level entries of a library (rows where parent_id IS NULL).
    pub fn titles(&self, _library_id: i64) -> Result<Vec<Entry>> {
        todo!()
    }
}
