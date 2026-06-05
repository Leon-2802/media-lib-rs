//! Service for querying and manipulating [`Entry`] records in the database.
//!
//! [`Entry`]: crate::database::models::data::Entry

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::types::Type;
use rusqlite::{Connection, OptionalExtension, Params, Result, Row, Statement};
use thiserror::Error;

use crate::database::models::data::{Entry, EntryKind, ItemType};

pub(crate) const ENTRY_COLUMNS: &str =
    "id, library_id, parent_id, path, kind, item_type, size, mtime";

/// SQL used to insert or refresh an entry by its `(library_id, path)` key.
pub(crate) const UPSERT_ENTRY_SQL: &str = "
    INSERT INTO entries (library_id, parent_id, path, kind, item_type, size, mtime)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
    ON CONFLICT(library_id, path) DO UPDATE SET
        parent_id = excluded.parent_id,
        kind = excluded.kind,
        item_type = excluded.item_type,
        size = excluded.size,
        mtime = excluded.mtime
    RETURNING id";

/// Errors converted into rusqlite errors by entry write helpers.
#[derive(Debug, Error)]
enum EntryWriteError {
    /// A filesystem path could not be represented as UTF-8 database text.
    #[error("path is not valid UTF-8: {0:?}")]
    NonUtf8Path(PathBuf),
    /// A requested parent relationship would break the entry tree.
    #[error("{0}")]
    InvalidParent(&'static str),
}

/// Writable entry state used by insert/update operations.
pub struct EntryWrite {
    /// Library that owns the entry.
    pub library_id: i64,
    /// Parent entry in the same library, or `None` for a root entry.
    ///
    /// The schema enforces this with a composite foreign key on
    /// `(library_id, parent_id)`.
    pub parent_id: Option<i64>,
    /// Absolute filesystem path stored for this entry.
    pub path: PathBuf,
    /// Whether the entry is a file or directory.
    pub kind: EntryKind,
    /// Media type inferred from the file extension, or `None` for folders and unknown files.
    pub item_type: Option<ItemType>,
    /// File size in bytes, or `None` for folders.
    pub size: Option<i64>,
    /// Last-modified Unix timestamp in seconds, or `None` when unavailable.
    pub mtime: Option<i64>,
}

impl EntryWrite {
    /// Builds writable entry state from filesystem scan metadata.
    pub fn scanned(
        library_id: i64,
        parent_id: Option<i64>,
        path: &Path,
        is_dir: bool,
        size: Option<i64>,
        mtime: Option<i64>,
    ) -> Self {
        let kind = if is_dir {
            EntryKind::Folder
        } else {
            EntryKind::File
        };
        let item_type = if is_dir {
            None
        } else {
            ItemType::from_path(path)
        };

        Self {
            library_id,
            parent_id,
            path: path.to_path_buf(),
            kind,
            item_type,
            size,
            mtime,
        }
    }
}

/// Service for managing file/folder entries within libraries.
pub struct EntryService {
    conn: Arc<Mutex<Connection>>,
}

impl EntryService {
    /// Creates a new `EntryService` backed by the given connection.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Locks the shared database connection for this service operation.
    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().expect("database mutex poisoned")
    }

    /// Fetches a single entry by its numeric `id`. Returns `None` if not found.
    pub fn get(&self, id: i64) -> Result<Option<Entry>> {
        let conn = self.conn();
        let sql = format!("SELECT {ENTRY_COLUMNS} FROM entries WHERE id = ?1");
        conn.query_row(&sql, [id], entry_from_row).optional()
    }

    /// Fetches a single entry by its library and absolute on-disk `path`.
    /// Returns `None` if not found.
    pub fn by_library_path(&self, library_id: i64, path: &Path) -> Result<Option<Entry>> {
        let path = path_to_str(path)?;
        let conn = self.conn();
        let sql =
            format!("SELECT {ENTRY_COLUMNS} FROM entries WHERE library_id = ?1 AND path = ?2");
        conn.query_row(&sql, (library_id, path), entry_from_row)
            .optional()
    }

    /// Returns all direct children (entries with `parent_id == given id`),
    /// ordered by path.
    pub fn children(&self, parent_id: i64) -> Result<Vec<Entry>> {
        let conn = self.conn();
        let sql = format!("SELECT {ENTRY_COLUMNS} FROM entries WHERE parent_id = ?1 ORDER BY path");
        query_entries(&conn, &sql, [parent_id])
    }

    /// Returns all recursive descendants of `root_id`, excluding the root entry
    /// itself, ordered by depth and then path. Uses a recursive CTE.
    pub fn descendants(&self, root_id: i64) -> Result<Vec<Entry>> {
        let conn = self.conn();
        query_entries(
            &conn,
            "
            WITH RECURSIVE subtree AS (
                SELECT
                    id,
                    library_id,
                    parent_id,
                    path,
                    kind,
                    item_type,
                    size,
                    mtime,
                    0 AS depth
                FROM entries
                WHERE id = ?1

                UNION ALL

                SELECT
                    e.id,
                    e.library_id,
                    e.parent_id,
                    e.path,
                    e.kind,
                    e.item_type,
                    e.size,
                    e.mtime,
                    s.depth + 1
                FROM entries e
                INNER JOIN subtree s
                    ON e.parent_id = s.id
            )
            SELECT
                id,
                library_id,
                parent_id,
                path,
                kind,
                item_type,
                size,
                mtime
            FROM subtree
            WHERE id != ?1
            ORDER BY depth ASC, path",
            [root_id],
        )
    }

    /// Returns the entry with `entry_id` and all of its ancestors up to the
    /// root (where `parent_id IS NULL`), ordered root first and entry last.
    /// Uses a recursive CTE.
    pub fn ancestors(&self, entry_id: i64) -> Result<Vec<Entry>> {
        let conn = self.conn();
        query_entries(
            &conn,
            "
            WITH RECURSIVE lineage AS (
                SELECT
                    id,
                    library_id,
                    parent_id,
                    path,
                    kind,
                    item_type,
                    size,
                    mtime,
                    0 AS depth
                FROM entries
                WHERE id = ?1

                UNION ALL

                SELECT
                    e.id,
                    e.library_id,
                    e.parent_id,
                    e.path,
                    e.kind,
                    e.item_type,
                    e.size,
                    e.mtime,
                    l.depth + 1
                FROM entries e
                INNER JOIN lineage l
                    ON l.parent_id = e.id
            )
            SELECT
                id,
                library_id,
                parent_id,
                path,
                kind,
                item_type,
                size,
                mtime
            FROM lineage
            ORDER BY depth DESC",
            [entry_id],
        )
    }

    /// Returns the root entry of the tree containing `entry_id`.
    /// Walks up the `parent_id` chain until reaching a node whose `parent_id` is `NULL`.
    ///
    /// Returns `None` if `entry_id` itself is not found.
    pub fn title_of(&self, entry_id: i64) -> Result<Option<Entry>> {
        let ancestors = self.ancestors(entry_id)?;
        Ok(ancestors.into_iter().next())
    }

    /// Inserts a new entry or updates an existing one matching on `library_id` and `path`.
    ///
    /// Parent relationships are constrained by the database schema: `parent_id`,
    /// when present, must point to an entry in the same library.
    ///
    /// Returns the id of the inserted or updated row.
    pub fn upsert_entry(&self, entry: &EntryWrite) -> Result<i64> {
        let conn = self.conn();
        validate_parent_update(&conn, entry)?;
        let mut stmt = conn.prepare(UPSERT_ENTRY_SQL)?;
        upsert_entry_stmt(&mut stmt, entry)
    }

    /// Deletes the entry with the given `id`. Returns `true` if a row was deleted.
    pub fn delete(&self, id: i64) -> Result<bool> {
        let conn = self.conn();
        let rows_affected = conn.execute("DELETE FROM entries WHERE id = ?1", [id])?;
        Ok(rows_affected > 0)
    }

    /// Deletes the entry at the given `path` within `library_id`.
    /// Returns `true` if a row was deleted.
    pub fn delete_by_library_path(&self, library_id: i64, path: &Path) -> Result<bool> {
        let path = path_to_str(path)?;
        let conn = self.conn();
        let rows_affected = conn.execute(
            "DELETE FROM entries WHERE library_id = ?1 AND path = ?2",
            (library_id, path),
        )?;
        Ok(rows_affected > 0)
    }
}

/// Converts a filesystem path into the UTF-8 text stored in SQLite.
///
/// Paths are persisted in `TEXT` columns, so non-UTF-8 paths are rejected instead
/// of being lossily encoded.
pub(crate) fn path_to_str(path: &Path) -> Result<&str> {
    path.to_str().ok_or_else(|| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(EntryWriteError::NonUtf8Path(
            path.to_path_buf(),
        )))
    })
}

/// Returns all entries belonging to a library, ordered by path.
pub(crate) fn entries_for_library(conn: &Connection, library_id: i64) -> Result<Vec<Entry>> {
    let sql = format!(
        "SELECT {ENTRY_COLUMNS}
            FROM entries
            WHERE library_id = ?1
            ORDER BY path"
    );
    query_entries(conn, &sql, [library_id])
}

/// Runs an entry-selecting query and maps each row into an [`Entry`].
pub(crate) fn query_entries<P>(conn: &Connection, sql: &str, params: P) -> Result<Vec<Entry>>
where
    P: Params,
{
    conn.prepare(sql)?
        .query_map(params, entry_from_row)?
        .collect()
}

/// Executes a prepared entry upsert statement and returns the affected row id.
pub(crate) fn upsert_entry_stmt(stmt: &mut Statement<'_>, entry: &EntryWrite) -> Result<i64> {
    let path = path_to_str(&entry.path)?;
    stmt.query_row(
        (
            entry.library_id,
            entry.parent_id,
            path,
            entry.kind.as_str(),
            entry.item_type.map(|it| it.as_str()),
            entry.size,
            entry.mtime,
        ),
        |row| row.get(0),
    )
}

/// Maps an `entries` row using [`ENTRY_COLUMNS`] order into an [`Entry`].
pub(crate) fn entry_from_row(row: &Row<'_>) -> Result<Entry> {
    let path: String = row.get(3)?;
    let kind: String = row.get(4)?;
    let item_type: Option<String> = row.get(5)?;

    Ok(Entry {
        id: row.get(0)?,
        library_id: row.get(1)?,
        parent_id: row.get(2)?,
        path: PathBuf::from(path),
        kind: kind.parse().map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(4, Type::Text, Box::new(err))
        })?,
        item_type: item_type
            .map(|item_type| {
                item_type.parse().map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(5, Type::Text, Box::new(err))
                })
            })
            .transpose()?,
        size: row.get(6)?,
        mtime: row.get(7)?,
    })
}

/// Rejects parent updates that would make an entry its own ancestor.
fn validate_parent_update(conn: &Connection, entry: &EntryWrite) -> Result<()> {
    let Some(parent_id) = entry.parent_id else {
        return Ok(());
    };

    let path = path_to_str(&entry.path)?;
    let existing_id = conn
        .query_row(
            "SELECT id FROM entries WHERE library_id = ?1 AND path = ?2",
            (entry.library_id, path),
            |row| row.get(0),
        )
        .optional()?;

    let Some(entry_id) = existing_id else {
        return Ok(());
    };

    if parent_id == entry_id {
        return Err(invalid_parent("entry cannot be its own parent"));
    }

    let creates_cycle: i64 = conn.query_row(
        "
        WITH RECURSIVE descendants(id) AS (
            SELECT id
            FROM entries
            WHERE library_id = ?1 AND parent_id = ?2

            UNION

            SELECT e.id
            FROM entries e
            INNER JOIN descendants d ON e.parent_id = d.id
            WHERE e.library_id = ?1
        )
        SELECT EXISTS(SELECT 1 FROM descendants WHERE id = ?3)
        ",
        (entry.library_id, entry_id, parent_id),
        |row| row.get(0),
    )?;

    if creates_cycle != 0 {
        return Err(invalid_parent(
            "entry cannot be moved below one of its descendants",
        ));
    }

    Ok(())
}

/// Builds the rusqlite error used for invalid parent relationships.
fn invalid_parent(message: &'static str) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(EntryWriteError::InvalidParent(message)))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::database::Database;
    use crate::database::models::data::LibraryKind;

    fn db_with_two_libraries() -> (Database, i64, i64) {
        let db = Database::open_in_memory().unwrap();
        let libraries = db.libraries();
        let library_id = libraries
            .add(
                "Library 1",
                &PathBuf::from("test-library-1"),
                LibraryKind::Movies,
            )
            .unwrap();
        let other_library_id = libraries
            .add(
                "Library 2",
                &PathBuf::from("test-library-2"),
                LibraryKind::Movies,
            )
            .unwrap();

        (db, library_id, other_library_id)
    }

    fn add_entry(
        entries: &EntryService,
        library_id: i64,
        parent_id: Option<i64>,
        path: &PathBuf,
        is_dir: bool,
    ) -> i64 {
        let entry = EntryWrite::scanned(
            library_id,
            parent_id,
            path,
            is_dir,
            if is_dir { None } else { Some(10) },
            Some(1),
        );
        entries.upsert_entry(&entry).unwrap()
    }

    #[test]
    fn by_library_path_scopes_duplicate_paths() {
        let (db, library_id, other_library_id) = db_with_two_libraries();
        let entries = db.entries();
        let shared_path = PathBuf::from("shared/movie.mkv");
        let entry = EntryWrite::scanned(library_id, None, &shared_path, false, Some(10), Some(1));
        let other_entry = EntryWrite::scanned(
            other_library_id,
            None,
            &shared_path,
            false,
            Some(20),
            Some(2),
        );

        let entry_id = entries.upsert_entry(&entry).unwrap();
        let other_entry_id = entries.upsert_entry(&other_entry).unwrap();

        assert_eq!(
            entries
                .by_library_path(library_id, &shared_path)
                .unwrap()
                .unwrap()
                .id,
            entry_id
        );
        assert_eq!(
            entries
                .by_library_path(other_library_id, &shared_path)
                .unwrap()
                .unwrap()
                .id,
            other_entry_id
        );
    }

    #[test]
    fn descendants_excludes_root() {
        let (db, library_id, _) = db_with_two_libraries();
        let entries = db.entries();
        let root_path = PathBuf::from("Library/Movie");
        let child_path = root_path.join("movie.mkv");
        let root_id = add_entry(&entries, library_id, None, &root_path, true);
        let child_id = add_entry(&entries, library_id, Some(root_id), &child_path, false);

        let descendants = entries.descendants(root_id).unwrap();

        assert_eq!(descendants.len(), 1);
        assert_eq!(descendants[0].id, child_id);
    }

    #[test]
    fn ancestors_are_ordered_root_first_and_title_uses_root() {
        let (db, library_id, _) = db_with_two_libraries();
        let entries = db.entries();
        let root_path = PathBuf::from("Library/Movie");
        let season_path = root_path.join("Season 1");
        let file_path = season_path.join("movie.mkv");
        let root_id = add_entry(&entries, library_id, None, &root_path, true);
        let season_id = add_entry(&entries, library_id, Some(root_id), &season_path, true);
        let file_id = add_entry(&entries, library_id, Some(season_id), &file_path, false);

        let ancestors = entries.ancestors(file_id).unwrap();

        assert_eq!(
            ancestors.iter().map(|entry| entry.id).collect::<Vec<_>>(),
            [root_id, season_id, file_id]
        );
        assert_eq!(entries.title_of(file_id).unwrap().unwrap().id, root_id);
        assert!(entries.title_of(i64::MAX).unwrap().is_none());
    }

    #[test]
    fn upsert_rejects_self_parent_cycle() {
        let (db, library_id, _) = db_with_two_libraries();
        let entries = db.entries();
        let path = PathBuf::from("Library/Movie");
        let entry = EntryWrite::scanned(library_id, None, &path, true, None, Some(1));
        let entry_id = entries.upsert_entry(&entry).unwrap();
        let self_parent =
            EntryWrite::scanned(library_id, Some(entry_id), &path, true, None, Some(2));

        assert!(entries.upsert_entry(&self_parent).is_err());
        assert_eq!(entries.get(entry_id).unwrap().unwrap().parent_id, None);
    }

    #[test]
    fn upsert_rejects_descendant_parent_cycle() {
        let (db, library_id, _) = db_with_two_libraries();
        let entries = db.entries();
        let root_path = PathBuf::from("Library/Movie");
        let child_path = root_path.join("Season 1");
        let root = EntryWrite::scanned(library_id, None, &root_path, true, None, Some(1));
        let root_id = entries.upsert_entry(&root).unwrap();
        let child =
            EntryWrite::scanned(library_id, Some(root_id), &child_path, true, None, Some(2));
        let child_id = entries.upsert_entry(&child).unwrap();
        let root_under_child =
            EntryWrite::scanned(library_id, Some(child_id), &root_path, true, None, Some(3));

        assert!(entries.upsert_entry(&root_under_child).is_err());
        assert_eq!(entries.get(root_id).unwrap().unwrap().parent_id, None);
    }

    #[test]
    fn cross_library_parent_is_rejected() {
        let (db, library_id, other_library_id) = db_with_two_libraries();
        let entries = db.entries();
        let parent_path = PathBuf::from("Library 1/Parent");
        let child_path = PathBuf::from("Library 2/Child");
        let parent = EntryWrite::scanned(library_id, None, &parent_path, true, None, Some(1));
        let parent_id = entries.upsert_entry(&parent).unwrap();
        let cross_library_child = EntryWrite::scanned(
            other_library_id,
            Some(parent_id),
            &child_path,
            false,
            Some(10),
            Some(2),
        );

        assert!(entries.upsert_entry(&cross_library_child).is_err());
    }

    #[test]
    fn delete_removes_entry_by_id() {
        let (db, library_id, _) = db_with_two_libraries();
        let entries = db.entries();
        let path = PathBuf::from("Library/Movie");
        let entry_id = add_entry(&entries, library_id, None, &path, true);

        assert!(entries.delete(entry_id).unwrap());
        assert!(entries.get(entry_id).unwrap().is_none());
        assert!(!entries.delete(entry_id).unwrap());
    }

    #[test]
    fn delete_by_library_path_scopes_duplicate_paths() {
        let (db, library_id, other_library_id) = db_with_two_libraries();
        let entries = db.entries();
        let shared_path = PathBuf::from("shared/movie.mkv");
        let entry_id = add_entry(&entries, library_id, None, &shared_path, false);
        let other_entry_id = add_entry(&entries, other_library_id, None, &shared_path, false);

        assert!(
            entries
                .delete_by_library_path(library_id, &shared_path)
                .unwrap()
        );
        assert!(entries.get(entry_id).unwrap().is_none());
        assert_eq!(
            entries.get(other_entry_id).unwrap().unwrap().path,
            shared_path
        );
    }
}
