//! Service for querying and manipulating [`Entry`] records in the database.
//!
//! [`Entry`]: crate::database::models::data::Entry

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

use crate::database::models::data::{Entry, EntryKind, ItemType};

/// Service for managing file/folder entries within libraries.
pub struct EntryService {
    conn: Arc<Mutex<Connection>>,
}

impl EntryService {
    /// Creates a new `EntryService` backed by the given connection.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Fetches a single entry by its numeric `id`. Returns `None` if not found.
    pub fn get(&self, id: i64) -> Result<Option<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, library_id, parent_id, name, path, kind, item_type, size, mtime FROM entries WHERE id = ?1"
        )?;
        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => {
                let path: String = row.get(4)?;
                let kind: String = row.get(5)?;
                let item_type: Option<String> = row.get(6)?;
                Ok(Some(Entry {
                    id: row.get(0)?,
                    library_id: row.get(1)?,
                    parent_id: row.get(2)?,
                    name: row.get(3)?,
                    path: PathBuf::from(path),
                    kind: EntryKind::from(kind).unwrap_or_default(),
                    item_type: item_type.and_then(ItemType::from),
                    size: row.get(7)?,
                    mtime: row.get(8)?,
                }))
            }
            None => Ok(None),
        }
    }

    /// Fetches a single entry by its absolute on-disk `path`. Returns `None` if not found.
    pub fn by_path(&self, path: &Path) -> Result<Option<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, library_id, parent_id, name, path, kind, item_type, size, mtime FROM entries WHERE path = ?1"
        )?;
        let mut rows = stmt.query([path.to_str()])?;
        match rows.next()? {
            Some(row) => {
                let path: String = row.get(4)?;
                let kind: String = row.get(5)?;
                let item_type: Option<String> = row.get(6)?;
                Ok(Some(Entry {
                    id: row.get(0)?,
                    library_id: row.get(1)?,
                    parent_id: row.get(2)?,
                    name: row.get(3)?,
                    path: PathBuf::from(path),
                    kind: EntryKind::from(kind).unwrap_or_default(),
                    item_type: item_type.and_then(ItemType::from),
                    size: row.get(7)?,
                    mtime: row.get(8)?,
                }))
            }
            None => Ok(None),
        }
    }

    /// Returns all direct children (entries with `parent_id == given id`).
    pub fn children(&self, parent_id: i64) -> Result<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, library_id, parent_id, name, path, kind, item_type, size, mtime FROM entries WHERE parent_id = ?1"
        )?;
        let mut rows = stmt.query([parent_id])?;
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

    /// Returns the entry with `id` and all of its recursive descendants,
    /// ordered by path. Uses a recursive CTE.
    ///
    /// Does **not** include the root entry itself — only its children.
    pub fn descendants(&self, root_id: i64) -> Result<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "
            WITH RECURSIVE subtree AS (
                SELECT
                    id,
                    library_id,
                    parent_id,
                    name,
                    path,
                    kind,
                    item_type,
                    size,
                    mtime
                FROM entries
                WHERE id = ?1

                UNION ALL

                SELECT
                    e.id,
                    e.library_id,
                    e.parent_id,
                    e.name,
                    e.path,
                    e.kind,
                    e.item_type,
                    e.size,
                    e.mtime
                FROM entries e
                INNER JOIN subtree s
                    ON e.parent_id = s.id
            )
            SELECT
                id,
                library_id,
                parent_id,
                name,
                path,
                kind,
                item_type,
                size,
                mtime
            FROM subtree
            ORDER BY path",
        )?;
        let mut rows = stmt.query([root_id])?;
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

    /// Returns the entry with `entry_id` and all of its ancestors up to the
    /// root (where `parent_id IS NULL`), ordered by path. Uses a recursive CTE.
    ///
    /// The first element of the returned vec is the root entry.
    pub fn ancestors(&self, entry_id: i64) -> Result<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "
            WITH RECURSIVE lineage AS (
                SELECT
                    id,
                    library_id,
                    parent_id,
                    name,
                    path,
                    kind,
                    item_type,
                    size,
                    mtime
                FROM entries
                WHERE id = ?1

                UNION ALL

                SELECT
                    e.id,
                    e.library_id,
                    e.parent_id,
                    e.name,
                    e.path,
                    e.kind,
                    e.item_type,
                    e.size,
                    e.mtime
                FROM entries e
                INNER JOIN lineage l
                    ON l.parent_id = e.id
            )
            SELECT
                id,
                library_id,
                parent_id,
                name,
                path,
                kind,
                item_type,
                size,
                mtime
            FROM lineage
            ORDER BY path",
        )?;
        let mut rows = stmt.query([entry_id])?;
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

    /// Returns the root entry of the tree containing `entry_id`.
    /// Walks up the `parent_id` chain until reaching a node whose `parent_id` is `NULL`.
    ///
    /// Returns `None` if `entry_id` itself is not found.
    pub fn title_of(&self, entry_id: i64) -> Result<Option<Entry>> {
        let ancestors = self.ancestors(entry_id)?;
        Ok(ancestors.first().cloned())
    }

    /// Inserts a new entry or updates an existing one matching on `path`.
    ///
    /// The `kind` is inferred from `path.is_dir()` and `item_type` is inferred
    /// from the file extension via [`ItemType::from_path`].
    ///
    /// Returns the rowid of the inserted or updated row.
    pub fn upsert_entry(
        &self,
        library_id: i64,
        parent_id: Option<i64>,
        name: &str,
        path: &Path,
        size: Option<i64>,
        mtime: Option<i64>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let kind = if path.is_dir() {
            EntryKind::Folder
        } else {
            EntryKind::File
        };
        conn.execute(
            "
                INSERT INTO entries (library_id, parent_id, name, path, kind, item_type, size, mtime)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                ON CONFLICT(path) DO UPDATE SET
                    library_id = excluded.library_id,
                    parent_id = excluded.parent_id,
                    name = excluded.name,
                    kind = excluded.kind,
                    item_type = excluded.item_type,
                    size = excluded.size,
                    mtime = excluded.mtime",
            (
                library_id,
                parent_id,
                name,
                path.to_str(),
                kind.as_str(),
                ItemType::from_path(path).map(|it| it.as_str()),
                size,
                mtime,
            ),
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Deletes the entry with the given `id`. Returns `true` if a row was deleted.
    pub fn delete(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM entries WHERE id = ?1", [id])?;
        Ok(rows_affected > 0)
    }
}

/// Returns `true` if `entry` is eligible for rating.
///
/// Folders are always rateable; image files (e.g. manga pages) are excluded.
pub fn is_rateable(entry: &Entry) -> bool {
    use crate::database::models::data::EntryKind;
    match entry.kind {
        EntryKind::Folder => true,
        EntryKind::File => !matches!(entry.item_type, Some(ItemType::Img)),
    }
}
