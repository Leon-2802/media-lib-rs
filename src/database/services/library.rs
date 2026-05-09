use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

use crate::database::models::data::{Entry, EntryKind, ItemType, Library, LibraryKind};

pub struct LibraryService {
    conn: Arc<Mutex<Connection>>,
}

impl LibraryService {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn add(&self, name: &str, path: &Path, kind: LibraryKind) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO library (name, path, kind) VALUES (?1, ?2, ?3)",
            (name, path.to_str(), kind.as_str()),
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get(&self, id: i64) -> Result<Option<Library>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, path, kind FROM library WHERE id = ?1")?;
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

    pub fn all(&self) -> Result<Vec<Library>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, path, kind FROM library")?;
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

    pub fn delete(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM library WHERE id = ?1", [id])?;
        Ok(rows_affected > 0)
    }

    /// Top-level entries of a library (rows where parent_id IS NULL).
    pub fn titles(&self, library_id: i64) -> Result<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("
            SELECT e.id, e.library_id, e.parent_id, e.name, e.path, e.kind, e.item_type, e.size, e.mtime
            FROM entries e
            JOIN library l ON e.id = l.id
            WHERE l.id = ?1
        ")?;
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
