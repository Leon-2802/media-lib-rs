use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

use crate::database::models::data::Entry;

pub struct FavoriteService {
    conn: Arc<Mutex<Connection>>,
}

impl FavoriteService {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn add(&self, _entry_id: i64) -> Result<()> {
        todo!()
    }

    pub fn remove(&self, _entry_id: i64) -> Result<bool> {
        todo!()
    }

    pub fn is_favorite(&self, _entry_id: i64) -> Result<bool> {
        todo!()
    }

    /// All favorited entries, joined through the favorites table.
    pub fn all(&self) -> Result<Vec<Entry>> {
        todo!()
    }
}
