//! # Database Module
//!
//! Provides the core database abstraction for media-lib-rs, wrapping a SQLite database
//! and exposing services for managing libraries, entries, tags, ratings, favorites,
//! search history, and library scanning.
//!
//! # Example
//!
//! ```
//! use std::path::PathBuf;
//! let db = database::Database::open(PathBuf::from("media.db").as_ref()).unwrap();
//! ```

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

use crate::database::services::search_history::SearchHistoryService;

/// Errors that can occur when opening or operating on the database.
#[derive(Debug, Error)]
pub enum DbError {
    /// Wraps a [`rusqlite::Error`] from the underlying SQLite connection.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    /// Wraps a [`rusqlite_migration::Error`] from schema migrations.
    #[error(transparent)]
    Migration(#[from] rusqlite_migration::Error),
}

/// The main database handle. Cloneable and safe to share across threads
/// via the internal `Arc<Mutex<Connection>>`.
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Opens (or creates) a SQLite database at the given path, running any pending
    /// migrations and enabling WAL mode and foreign key enforcement.
    pub fn open(path: &Path) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        Self::configure(conn)
    }

    /// Creates a new in-memory SQLite database. Useful for tests.
    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        Self::configure(conn)
    }

    /// Applies pragma settings and runs migrations. Called by both [`Self::open`]
    /// and [`Self::open_in_memory`].
    fn configure(mut conn: Connection) -> Result<Self, DbError> {
        // Pragmas must run before any transaction; migrations open their own.
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
        migrations::migrations().to_latest(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Returns a [`LibraryService`] for managing library records.
    pub fn libraries(&self) -> LibraryService {
        LibraryService::new(self.conn.clone())
    }

    /// Returns an [`EntryService`] for querying and manipulating file/folder entries.
    pub fn entries(&self) -> EntryService {
        EntryService::new(self.conn.clone())
    }

    /// Returns a [`TagService`] for managing tags and their associations.
    pub fn tags(&self) -> TagService {
        TagService::new(self.conn.clone())
    }

    /// Returns a [`RatingService`] for managing per-entry ratings.
    pub fn ratings(&self) -> RatingService {
        RatingService::new(self.conn.clone())
    }

    /// Returns a [`FavoriteService`] for managing favorited entries.
    pub fn favorites(&self) -> FavoriteService {
        FavoriteService::new(self.conn.clone())
    }

    /// Returns a [`SearchHistoryService`] for tracking search queries.
    pub fn search_history(&self) -> SearchHistoryService {
        SearchHistoryService::new(self.conn.clone())
    }

    /// Returns a [`ScanService`] for scanning library directories and updating the database.
    pub fn scanner(&self) -> ScanService {
        ScanService::new(self.conn.clone())
    }
}
