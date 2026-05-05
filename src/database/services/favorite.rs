use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use rusqlite::{Connection, Result};

use crate::database::models::data::{Entry, EntryKind, ItemType};

pub struct FavoriteService {
    conn: Arc<Mutex<Connection>>,
}

impl FavoriteService {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn add(&self, entry_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO favorites (entry_id) VALUES (?1)",
            [entry_id],
        )?;
        Ok(())
    }

    pub fn remove(&self, entry_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected =
            conn.execute("DELETE FROM favorites WHERE entry_id = ?1", [entry_id])?;
        Ok(rows_affected > 0)
    }

    pub fn is_favorite(&self, entry_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT 1 FROM favorites WHERE entry_id = ?1")?;
        let mut rows = stmt.query([entry_id])?;
        Ok(rows.next()?.is_some())
    }

    /// All favorited entries, joined through the favorites table.
    pub fn all(&self) -> Result<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("
            SELECT e.id, e.library_id, e.parent_id, e.name, e.path, e.kind, e.item_type, e.size, e.mtime
            FROM entries e
            JOIN favorites f ON e.id = f.entry_id
        ")?;
        let mut rows = stmt.query([])?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            let path: String = row.get(4)?;
            let kind: String = row.get(5)?;
            let item_type: Option<String> = row.get(6)?;

            entries.push(Entry {
                id: row.get(0)?,
                library_id: row.get(1)?,
                parent_id: row.get(2)?,
                name: row.get(3)?,

                path: PathBuf::from(path),
                kind: EntryKind::from(kind).unwrap_or_default(),
                item_type: item_type.and_then(ItemType::from),

                size: row.get(7)?,
                mtime: row.get(8)?,
            });
        }

        Ok(entries)
    }
}
