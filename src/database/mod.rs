pub mod migrations;
pub mod models;
pub mod services;

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use thiserror::Error;

use services::entry::EntryService;
use services::favorite::FavoriteService;
use services::library::LibraryService;
use services::rating::RatingService;
use services::scan::ScanService;
use services::tag::TagService;

#[derive(Debug, Error)]
pub enum DbError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Migration(#[from] rusqlite_migration::Error),
}

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        Self::configure(conn)
    }

    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        Self::configure(conn)
    }

    fn configure(mut conn: Connection) -> Result<Self, DbError> {
        // Pragmas must run before any transaction; migrations open their own.
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
        migrations::migrations().to_latest(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn libraries(&self) -> LibraryService {
        LibraryService::new(self.conn.clone())
    }

    pub fn entries(&self) -> EntryService {
        EntryService::new(self.conn.clone())
    }

    pub fn tags(&self) -> TagService {
        TagService::new(self.conn.clone())
    }

    pub fn ratings(&self) -> RatingService {
        RatingService::new(self.conn.clone())
    }

    pub fn favorites(&self) -> FavoriteService {
        FavoriteService::new(self.conn.clone())
    }

    pub fn scanner(&self) -> ScanService {
        ScanService::new(self.conn.clone())
    }
}
