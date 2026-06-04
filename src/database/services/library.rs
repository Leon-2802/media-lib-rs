//! Service for managing [`Library`] records in the database.
//!
//! [`Library`]: crate::database::models::data::Library

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::types::Type;
use rusqlite::{Connection, OptionalExtension, Result, Row};

use crate::database::models::data::{Entry, Library, LibraryKind};
use crate::database::services::entry::{
    ENTRY_COLUMNS, entries_for_library, path_to_str, query_entries,
};

/// Service for registering and removing media libraries.
pub struct LibraryService {
    conn: Arc<Mutex<Connection>>,
}

impl LibraryService {
    /// Creates a new `LibraryService` backed by the given connection.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Locks the shared database connection for this service operation.
    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().expect("database mutex poisoned")
    }

    /// Registers a new library and returns its assigned `id`.
    ///
    /// # Arguments
    ///
    /// * `name` — human-readable label (e.g. "Movies", "My Manga")
    /// * `path` — absolute path to the library root on disk
    /// * `kind` — the category of this library
    pub fn add(&self, name: &str, path: &Path, kind: LibraryKind) -> Result<i64> {
        let path = path_to_str(path)?;
        let conn = self.conn();
        conn.execute(
            "INSERT INTO libraries (name, path, kind) VALUES (?1, ?2, ?3)",
            (name, path, kind.as_str()),
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Fetches a single library by its numeric `id`. Returns `None` if not found.
    pub fn get(&self, id: i64) -> Result<Option<Library>> {
        let conn = self.conn();
        library_by_id(&conn, id)
    }

    /// Returns all registered libraries, ordered by name.
    pub fn all(&self) -> Result<Vec<Library>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, name, path, kind FROM libraries")?;
        let mut rows = stmt.query([])?;
        let mut libraries = Vec::new();
        while let Some(row) = rows.next()? {
            libraries.push(library_from_row(row)?);
        }

        Ok(libraries)
    }

    /// Deletes the library with the given `id`. Returns `true` if a row was deleted.
    pub fn delete(&self, id: i64) -> Result<bool> {
        let conn = self.conn();
        let rows_affected = conn.execute("DELETE FROM libraries WHERE id = ?1", [id])?;
        Ok(rows_affected > 0)
    }

    /// Returns all root-level entries of the library (those with `parent_id IS NULL`),
    /// ordered by path. These are the top-level "titles" in the library hierarchy.
    pub fn titles(&self, library_id: i64) -> Result<Vec<Entry>> {
        let conn = self.conn();
        let sql = format!(
            "SELECT {ENTRY_COLUMNS}
                FROM entries
                WHERE library_id = ?1 AND parent_id IS NULL
                ORDER BY path"
        );
        query_entries(&conn, &sql, [library_id])
    }

    /// Returns all entries belonging to the library, ordered by path.
    pub fn get_all_entries(&self, library_id: i64) -> Result<Vec<Entry>> {
        let conn = self.conn();
        entries_for_library(&conn, library_id)
    }
}

/// Fetches a library by id using an existing database connection.
pub(crate) fn library_by_id(conn: &Connection, id: i64) -> Result<Option<Library>> {
    conn.query_row(
        "SELECT id, name, path, kind FROM libraries WHERE id = ?1",
        [id],
        library_from_row,
    )
    .optional()
}

/// Maps a `libraries` row into a [`Library`].
pub(crate) fn library_from_row(row: &Row<'_>) -> Result<Library> {
    let path: String = row.get(2)?;
    let kind: String = row.get(3)?;

    Ok(Library {
        id: row.get(0)?,
        name: row.get(1)?,
        path: PathBuf::from(path),
        kind: kind.parse().map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(3, Type::Text, Box::new(err))
        })?,
    })
}
