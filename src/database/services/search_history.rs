//! Service for tracking user search history.

use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

use crate::database::models::app::SearchHistory;

/// Service for recording and retrieving past search queries.
pub struct SearchHistoryService {
    conn: Arc<Mutex<Connection>>,
}

impl SearchHistoryService {
    /// Creates a new `SearchHistoryService` backed by the given connection.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Records a search query with its timestamp `at` (Unix seconds).
    pub fn add(&self, query: &str, at: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO search_history (query, at) VALUES (?1, ?2)",
            (query, at),
        )?;
        Ok(())
    }

    /// Fetches a single search history record by `id`. Returns `None` if not found.
    pub fn get(&self, id: i64) -> Result<Option<SearchHistory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, query, at FROM search_history WHERE id = ?1")?;
        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => Ok(Some(SearchHistory {
                id: row.get(0)?,
                query: row.get(1)?,
                at: row.get(2)?,
            })),
            None => Ok(None),
        }
    }

    /// Deletes a single search history record by `id`. Returns `true` if a row was deleted.
    pub fn delete(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM search_history WHERE id = ?1", [id])?;
        Ok(rows_affected > 0)
    }

    /// Clears all search history. Returns `true` if at least one row was deleted.
    pub fn clear(&self) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM search_history", [])?;
        Ok(rows_affected > 0)
    }

    /// Returns the `limit` most recent search records, ordered by timestamp descending.
    pub fn latest(&self, limit: i64) -> Result<Vec<SearchHistory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, query, at FROM search_history ORDER BY at DESC LIMIT ?1")?;
        let mut rows = stmt.query([limit])?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            entries.push(SearchHistory {
                id: row.get(0)?,
                query: row.get(1)?,
                at: row.get(2)?,
            })
        }
        Ok(entries)
    }
}
