//! Service for creating and managing tags, and attaching them to entries.

use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::Connection;

use crate::database::models::app::Tag;
use crate::database::models::data::Entry;
use crate::database::services::entry::{ENTRY_COLUMNS, query_entries};

/// Service for managing tags and their associations with entries.
pub struct TagService {
    conn: Arc<Mutex<Connection>>,
}

impl TagService {
    /// Creates a new `TagService` backed by the given connection.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Locks the shared database connection for this service operation.
    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().expect("database mutex poisoned")
    }

    /// Looks up a tag by name, creating it if it does not yet exist.
    /// Returns the tag's `id`.
    pub fn get_or_create(&self, name: &str) -> rusqlite::Result<i64> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO tags (name) VALUES (?1) ON CONFLICT(name) DO NOTHING",
            [name],
        )?;
        conn.query_row("SELECT id FROM tags WHERE name = ?1", [name], |row| {
            row.get(0)
        })
    }

    /// Attaches an existing tag to an entry. Idempotent (uses `INSERT OR IGNORE`).
    pub fn attach(&self, entry_id: i64, tag_id: i64) -> rusqlite::Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
            [entry_id, tag_id],
        )?;
        Ok(())
    }

    /// Removes the association between an entry and a tag.
    /// Returns `Ok(())` even if the association did not exist.
    pub fn detach(&self, entry_id: i64, tag_id: i64) -> rusqlite::Result<()> {
        let conn = self.conn();
        conn.execute(
            "DELETE FROM entry_tags WHERE entry_id = ?1 AND tag_id = ?2",
            [entry_id, tag_id],
        )?;
        Ok(())
    }

    /// Returns all tags attached to a given entry, ordered by name.
    pub fn for_entry(&self, entry_id: i64) -> rusqlite::Result<Vec<Tag>> {
        let conn = self.conn();
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

    /// Returns all entries that carry the given tag.
    pub fn entries_with_tag(&self, tag_id: i64) -> rusqlite::Result<Vec<Entry>> {
        let conn = self.conn();
        let sql = format!(
            "SELECT {ENTRY_COLUMNS}
                FROM entries e
                INNER JOIN entry_tags et ON et.entry_id = e.id
                WHERE et.tag_id = ?1"
        );
        query_entries(&conn, &sql, [tag_id])
    }
}
