use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

pub struct RatingService {
    conn: Arc<Mutex<Connection>>,
}

impl RatingService {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn set(&self, entry_id: i64, rating: i8) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO ratings (entry_id, rating) VALUES (?1, ?2) 
                      ON CONFLICT(entry_id) DO UPDATE SET rating=excluded.rating",
            [entry_id],
        )?;
        Ok(())
    }

    pub fn get(&self, entry_id: i64) -> Result<Option<i8>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT rating FROM ratings WHERE entry_id = ?1")?;
        let mut rows = stmt.query([entry_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn clear(&self, entry_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM ratings WHERE entry_id = ?1", [entry_id])?;
        Ok(rows_affected > 0)
    }
}
