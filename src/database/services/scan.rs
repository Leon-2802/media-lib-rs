//! Service for scanning a library directory and synchronizing the database.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use thiserror::Error;

/// Errors that can occur during a library scan.
#[derive(Debug, Error)]
pub enum ScanError {
    /// The requested library does not exist in the database.
    #[error("library not found")]
    LibraryNotFound,
    /// The library root path does not exist or is not readable.
    #[error("library root is missing or unreadable")]
    RootMissing,
    /// Another scan is already in progress (reserved for future use).
    #[error("another scan is in progress")]
    Busy,
    /// Wraps an I/O error encountered while walking the directory tree.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Wraps a database error encountered during the scan.
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
}

/// A non-fatal warning that occurred while processing a single file during a scan.
#[derive(Debug, Clone)]
pub struct ScanWarning {
    /// The path of the file that triggered the warning.
    pub path: PathBuf,
    /// A human-readable description of the warning.
    pub reason: String,
}

/// Summary of a completed scan operation. All counts reflect entries processed
/// in a single atomic transaction.
#[derive(Debug, Default, Clone)]
pub struct ScanReport {
    /// Number of new entries inserted into the database.
    pub inserted: usize,
    /// Number of existing entries that were updated.
    pub updated: usize,
    /// Number of entries deleted from the database.
    pub deleted: usize,
    /// Number of entries skipped (e.g. hidden files, unsupported extensions).
    pub skipped: usize,
    /// Per-file warnings that occurred during the scan.
    pub errors: Vec<ScanWarning>,
}

/// Service for walking a library directory tree and syncing it to the database.
pub struct ScanService {
    conn: Arc<Mutex<Connection>>,
}

impl ScanService {
    /// Creates a new `ScanService` backed by the given connection.
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Walks the library root, diffs against the existing database entries,
    /// and applies deletes, inserts, and updates in a single `BEGIN IMMEDIATE`
    /// transaction.
    ///
    /// Returns a [`ScanReport`] with counts of inserted, updated, deleted,
    /// skipped, and errored entries.
    ///
    /// # Errors
    ///
    /// Returns [`ScanError::LibraryNotFound`] if the library does not exist,
    /// or [`ScanError::RootMissing`] if the path is not accessible.
    pub fn scan_library(&self, _library_id: i64) -> Result<ScanReport, ScanError> {
        todo!()
    }
}

/// Internal representation of a file or directory on disk.
#[derive(Debug, Clone)]
struct DiskNode {
    path: PathBuf,
    is_dir: bool,
    size: Option<i64>,
    mtime: Option<i64>,
}

/// Recursively walks `root`, returning all files and directories encountered.
/// Hidden files (starting with `.`), dotfiles, and symlinks are skipped.
fn walk_dir(_root: &std::path::Path) -> Result<Vec<DiskNode>, ScanError> {
    todo!()
}

/// Holds the result of comparing the on-disk state against the database state.
#[derive(Debug, Default)]
struct Diff {
    /// Nodes present on disk but not in the database — should be inserted.
    to_insert: Vec<DiskNode>,
    /// Nodes present in both but with different size/mtime — should be updated.
    to_update: Vec<DiskNode>,
    /// Paths present in the database but not on disk — should be deleted.
    to_delete: Vec<PathBuf>,
}

/// Computes the diff between the existing database entries and the on-disk state.
fn diff(_existing: &HashMap<PathBuf, EntryRow>, _on_disk: &[DiskNode]) -> Diff {
    todo!()
}

/// Partial entry data loaded from the database for diff purposes.
#[derive(Debug, Clone)]
struct EntryRow {
    id: i64,
    parent_id: Option<i64>,
    size: Option<i64>,
    mtime: Option<i64>,
}
