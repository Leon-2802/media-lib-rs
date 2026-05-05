use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

use crate::database::models::app::Tag;
use crate::database::models::data::Entry;

pub struct TagService {
    conn: Arc<Mutex<Connection>>,
}

impl TagService {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn create(&self, _name: &str) -> Result<i64> {
        todo!()
    }

    pub fn get_or_create(&self, _name: &str) -> Result<i64> {
        todo!()
    }

    pub fn attach(&self, _entry_id: i64, _tag_id: i64) -> Result<()> {
        todo!()
    }

    pub fn detach(&self, _entry_id: i64, _tag_id: i64) -> Result<()> {
        todo!()
    }

    pub fn for_entry(&self, _entry_id: i64) -> Result<Vec<Tag>> {
        todo!()
    }

    pub fn entries_with_tag(&self, _tag_id: i64) -> Result<Vec<Entry>> {
        todo!()
    }
}
