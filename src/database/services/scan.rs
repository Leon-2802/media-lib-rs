use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("library not found")]
    LibraryNotFound,
    #[error("library root is missing or unreadable")]
    RootMissing,
    #[error("another scan is in progress")]
    Busy,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
}

#[derive(Debug, Clone)]
pub struct ScanWarning {
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Debug, Default, Clone)]
pub struct ScanReport {
    pub inserted: usize,
    pub updated: usize,
    pub deleted: usize,
    pub skipped: usize,
    pub errors: Vec<ScanWarning>,
}

pub struct ScanService {
    conn: Arc<Mutex<Connection>>,
}

impl ScanService {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Walk the library, diff against DB, apply deletes/inserts/updates in one
    /// `BEGIN IMMEDIATE` transaction. Returns counts and per-file warnings.
    pub fn scan_library(&self, _library_id: i64) -> Result<ScanReport, ScanError> {
        todo!()
    }
}

#[derive(Debug, Clone)]
struct DiskNode {
    path: PathBuf,
    is_dir: bool,
    size: Option<i64>,
    mtime: Option<i64>,
}

/// Walks `root` recursively. Skips hidden files, dotfiles, and symlinks.
fn walk_dir(_root: &std::path::Path) -> Result<Vec<DiskNode>, ScanError> {
    todo!()
}

#[derive(Debug, Default)]
struct Diff {
    to_insert: Vec<DiskNode>,
    to_update: Vec<DiskNode>,
    to_delete: Vec<PathBuf>,
}

fn diff(_existing: &HashMap<PathBuf, EntryRow>, _on_disk: &[DiskNode]) -> Diff {
    todo!()
}

#[derive(Debug, Clone)]
struct EntryRow {
    id: i64,
    parent_id: Option<i64>,
    size: Option<i64>,
    mtime: Option<i64>,
}
