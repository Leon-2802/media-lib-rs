use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

use crate::database::models::app::SearchHistory;

pub struct SearchHistoryService {
    conn: Arc<Mutex<Connection>>,
}

impl SearchHistoryService {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn add(&self, query: &str, at: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO search_history (query, at) VALUES (?1, ?2)",
            (query, at),
        )?;
        Ok(())
    }

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

    pub fn delete(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM search_history WHERE id = ?1", [id])?;
        Ok(rows_affected > 0)
    }

    pub fn clear(&self) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute("DELETE FROM search_history", [])?;
        Ok(rows_affected > 0)
    }

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
