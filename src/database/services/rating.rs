//! Service for assigning and removing numeric ratings on entries.

use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

/// Service for storing per-entry numeric ratings.
pub struct RatingService {
    conn: Arc<Mutex<Connection>>,
}

impl RatingService {
    /// Creates a new `RatingService` backed by the given connection.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Sets (or updates) the rating for `entry_id`. Uses `INSERT ... ON CONFLICT`
    /// to make the operation idempotent.
    ///
    /// `rating` is stored as a signed 8-bit integer (typically 1–5).
    pub fn set(&self, entry_id: i64, rating: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO ratings (entry_id, rating) VALUES (?1, ?2)
                      ON CONFLICT(entry_id) DO UPDATE SET rating=excluded.rating",
            [entry_id, rating],
        )?;
        Ok(())
    }

    /// Returns the rating for `entry_id`, or `None` if no rating has been set.
    pub fn get(&self, entry_id: i64) -> Result<Option<i64>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT rating FROM ratings WHERE entry_id = ?1")?;
        let mut rows = stmt.query([entry_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    /// Removes the rating for `entry_id`. Returns `true` if a row was deleted.
    pub fn delete(&self, entry_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM ratings WHERE entry_id = ?1", [entry_id])?;
        Ok(rows_affected > 0)
    }
}
