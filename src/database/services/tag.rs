use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

use crate::database::models::app::Tag;
use crate::database::models::data::{Entry, EntryKind, ItemType};

pub struct TagService {
    conn: Arc<Mutex<Connection>>,
}

impl TagService {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn get_or_create(&self, name: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO tags (name) VALUES (?1) ON CONFLICT(name) DO NOTHING",
            [name],
        )?;
        conn.query_row("SELECT id FROM tags WHERE name = ?1", [name], |row| {
            row.get(0)
        })
    }

    pub fn attach(&self, entry_id: i64, tag_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
            [entry_id, tag_id],
        )?;
        Ok(())
    }

    pub fn detach(&self, entry_id: i64, tag_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM entry_tags WHERE entry_id = ?1 AND tag_id = ?2",
            [entry_id, tag_id],
        )?;
        Ok(())
    }

    pub fn for_entry(&self, entry_id: i64) -> Result<Vec<Tag>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name FROM tags t
            INNER JOIN entry_tags et ON et.tag_id = t.id
            WHERE et.entry_id = ?1 ORDER BY t.name",
        )?;
        let mut rows = stmt.query([entry_id])?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            entries.push(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            });
        }

        Ok(entries)
    }

    pub fn entries_with_tag(&self, tag_id: i64) -> Result<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT e.id, e.library_id, e.parent_id, e.name, e.path, e.kind, e.item_type, e.size, e.mtime
                FROM entries e
                INNER JOIN entry_tags et ON et.entry_id = e.id
                WHERE et.tag_id = ?1"
        )?;
        let mut rows = stmt.query([tag_id])?;
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
