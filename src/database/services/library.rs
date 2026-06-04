//! Service for managing [`Library`] records in the database.
//!
//! [`Library`]: crate::database::models::data::Library

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

use crate::database::models::data::{Entry, EntryKind, ItemType, Library, LibraryKind};

/// Service for registering and removing media libraries.
pub struct LibraryService {
    conn: Arc<Mutex<Connection>>,
}

impl LibraryService {
    /// Creates a new `LibraryService` backed by the given connection.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Registers a new library and returns its assigned `id`.
    ///
    /// # Arguments
    ///
    /// * `name` — human-readable label (e.g. "Movies", "My Manga")
    /// * `path` — absolute path to the library root on disk
    /// * `kind` — the category of this library
    pub fn add(&self, name: &str, path: &Path, kind: LibraryKind) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO libraries (name, path, kind) VALUES (?1, ?2, ?3)",
            (name, path.to_str(), kind.as_str()),
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Fetches a single library by its numeric `id`. Returns `None` if not found.
    pub fn get(&self, id: i64) -> Result<Option<Library>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, path, kind FROM libraries WHERE id = ?1")?;
        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => {
                let path: String = row.get(2)?;
                let kind: String = row.get(3)?;
                Ok(Some(Library {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: PathBuf::from(path),
                    kind: LibraryKind::from(kind).unwrap_or_default(),
                }))
            }
            None => Ok(None),
        }
    }

    /// Returns all registered libraries, ordered by name.
    pub fn all(&self) -> Result<Vec<Library>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, path, kind FROM libraries")?;
        let mut rows = stmt.query([])?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            let path: String = row.get(2)?;
            let kind: String = row.get(3)?;
            entries.push(Library {
                id: row.get(0)?,
                name: row.get(1)?,
                path: PathBuf::from(path),
                kind: LibraryKind::from(kind).unwrap_or_default(),
            });
        }

        Ok(entries)
    }

    /// Deletes the library with the given `id`. Returns `true` if a row was deleted.
    pub fn delete(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM libraries WHERE id = ?1", [id])?;
        Ok(rows_affected > 0)
    }

    /// Returns all root-level entries of the library (those with `parent_id IS NULL`),
    /// ordered by name. These are the top-level "titles" in the library hierarchy.
    pub fn titles(&self, library_id: i64) -> Result<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, library_id, parent_id, name, path, kind, item_type, size, mtime
                FROM entries
                WHERE library_id = ?1 AND parent_id IS NULL
                ORDER BY name",
        )?;
        let mut rows = stmt.query([library_id])?;
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
