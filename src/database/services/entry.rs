use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

use crate::database::models::data::{Entry, ItemType};

pub struct EntryService {
    conn: Arc<Mutex<Connection>>,
}

impl EntryService {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn get(&self, _id: i64) -> Result<Option<Entry>> {
        todo!()
    }

    pub fn by_path(&self, _library_id: i64, _path: &Path) -> Result<Option<Entry>> {
        todo!()
    }

    pub fn children(&self, _parent_id: i64) -> Result<Vec<Entry>> {
        todo!()
    }

    /// Recursive CTE walking the subtree rooted at `root_id`.
    pub fn descendants(&self, _root_id: i64) -> Result<Vec<Entry>> {
        todo!()
    }

    /// Recursive CTE walking up to the root (parent_id IS NULL).
    pub fn ancestors(&self, _entry_id: i64) -> Result<Vec<Entry>> {
        todo!()
    }

    /// Walks up parent_id chain until parent_id IS NULL.
    pub fn title_of(&self, _entry_id: i64) -> Result<Entry> {
        todo!()
    }

    pub fn upsert_file(
        &self,
        _library_id: i64,
        _parent_id: Option<i64>,
        _name: &str,
        _path: &Path,
        _item_type: Option<ItemType>,
        _size: i64,
        _mtime: i64,
    ) -> Result<i64> {
        todo!()
    }

    pub fn upsert_folder(
        &self,
        _library_id: i64,
        _parent_id: Option<i64>,
        _name: &str,
        _path: &Path,
    ) -> Result<i64> {
        todo!()
    }

    pub fn delete(&self, _id: i64) -> Result<bool> {
        todo!()
    }
}

/// Folders are always rateable; image files (manga pages) are not.
pub fn is_rateable(entry: &Entry) -> bool {
    use crate::database::models::data::EntryKind;
    match entry.kind {
        EntryKind::Folder => true,
        EntryKind::File => !matches!(entry.item_type, Some(ItemType::Img)),
    }
}
