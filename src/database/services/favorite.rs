//! Service for managing favorited entries.

use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{Connection, Result};

use crate::database::models::data::Entry;
use crate::database::services::entry::{ENTRY_COLUMNS, query_entries};

/// Service for marking and querying favorite entries.
pub struct FavoriteService {
    conn: Arc<Mutex<Connection>>,
}

impl FavoriteService {
    /// Creates a new `FavoriteService` backed by the given connection.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Locks the shared database connection for this service operation.
    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().expect("database mutex poisoned")
    }

    /// Marks an entry as a favorite. Idempotent via `INSERT OR IGNORE`.
    pub fn add(&self, entry_id: i64) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT OR IGNORE INTO favorites (entry_id, added_at) VALUES (?1, unixepoch())",
            [entry_id],
        )?;
        Ok(())
    }

    /// Removes the favorite status of an entry. Returns `true` if a row was deleted.
    pub fn remove(&self, entry_id: i64) -> Result<bool> {
        let conn = self.conn();
        let rows_affected =
            conn.execute("DELETE FROM favorites WHERE entry_id = ?1", [entry_id])?;
        Ok(rows_affected > 0)
    }

    /// Returns `true` if the entry is currently marked as a favorite.
    pub fn is_favorite(&self, entry_id: i64) -> Result<bool> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT 1 FROM favorites WHERE entry_id = ?1")?;
        let mut rows = stmt.query([entry_id])?;
        Ok(rows.next()?.is_some())
    }

    /// Returns all favorited entries, joined from the `favorites` and `entries` tables.
    pub fn all(&self) -> Result<Vec<Entry>> {
        let conn = self.conn();
        let sql = format!(
            "
            SELECT {ENTRY_COLUMNS}
            FROM entries e
            JOIN favorites f ON e.id = f.entry_id
        "
        );
        query_entries(&conn, &sql, [])
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::database::Database;
    use crate::database::models::data::LibraryKind;
    use crate::database::services::entry::EntryWrite;

    #[test]
    fn add_sets_added_at_and_is_idempotent() {
        let db = Database::open_in_memory().unwrap();
        let library_id = db
            .libraries()
            .add("Library", &PathBuf::from("library"), LibraryKind::Movies)
            .unwrap();
        let path = PathBuf::from("library/movie.mkv");
        let entry = EntryWrite::scanned(library_id, None, &path, false, Some(10), Some(1));
        let entry_id = db.entries().upsert_entry(&entry).unwrap();

        let favorites = db.favorites();
        favorites.add(entry_id).unwrap();
        favorites.add(entry_id).unwrap();

        let added_at: i64 = favorites
            .conn()
            .query_row(
                "SELECT added_at FROM favorites WHERE entry_id = ?1",
                [entry_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(favorites.is_favorite(entry_id).unwrap());
        assert!(added_at > 0);
        assert_eq!(favorites.all().unwrap().len(), 1);
    }
}
