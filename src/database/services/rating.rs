use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

pub struct RatingService {
    conn: Arc<Mutex<Connection>>,
}

impl RatingService {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn set(&self, _entry_id: i64, _rating: i8) -> Result<()> {
        todo!()
    }

    pub fn get(&self, _entry_id: i64) -> Result<Option<i8>> {
        todo!()
    }

    pub fn clear(&self, _entry_id: i64) -> Result<bool> {
        todo!()
    }
}
