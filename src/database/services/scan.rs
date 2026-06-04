//! Service for scanning a library directory and synchronizing the database.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::UNIX_EPOCH;
use std::{fs, io};

use rusqlite::{Connection, TransactionBehavior};
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

use crate::database::models::data::Entry;
use crate::database::services::entry::{
    EntryWrite, UPSERT_ENTRY_SQL, entries_for_library, upsert_entry_stmt,
};
use crate::database::services::library::library_by_id;

/// Errors that can occur during a library scan.
#[derive(Debug, Error)]
pub enum ScanError {
    /// The library record does not exist.
    #[error("library not found")]
    LibraryNotFound,
    /// The configured library root cannot be scanned.
    #[error("invalid library root {path:?}: {reason}")]
    InvalidRoot {
        /// Configured library root path.
        path: PathBuf,
        /// Static reason suitable for logs and user-facing errors.
        reason: &'static str,
    },
    /// A scanned path cannot be related to the library hierarchy.
    #[error("invalid scanned path {path:?}: {reason}")]
    InvalidPath {
        /// Path from the current filesystem scan.
        path: PathBuf,
        /// Static reason suitable for logs and user-facing errors.
        reason: &'static str,
    },
    /// Wraps an I/O error encountered while walking the directory tree.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Wraps a database error encountered during the scan.
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
    /// Wraps a walkdir error encountered during directory tree traversal.
    #[error(transparent)]
    WalkDir(#[from] walkdir::Error),
    /// File metadata could not be converted into the database representation.
    #[error("invalid metadata for {path:?}: {reason}")]
    InvalidMetadata {
        /// Path whose metadata could not be stored.
        path: PathBuf,
        /// Static reason suitable for logs and user-facing errors.
        reason: &'static str,
    },
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
}

/// Internal representation of a file or directory on disk.
#[derive(Debug, Clone)]
struct DiskNode {
    path: PathBuf,
    is_dir: bool,
    size: Option<i64>,
    mtime: Option<i64>,
}

impl DiskNode {
    /// Converts filesystem metadata into the write shape used by entry upserts.
    fn entry_write(&self, library_id: i64, parent_id: Option<i64>) -> EntryWrite {
        EntryWrite::scanned(
            library_id,
            parent_id,
            &self.path,
            self.is_dir,
            self.size,
            self.mtime,
        )
    }
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

    /// Locks the shared database connection for this service operation.
    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().expect("database mutex poisoned")
    }

    /// Walks the library root, diffs against the existing database entries,
    /// and applies deletes, inserts, and updates in a single `BEGIN IMMEDIATE`
    /// transaction.
    ///
    /// Returns a [`ScanReport`] with counts of inserted, updated and deleted entries.
    ///
    /// # Errors
    ///
    /// Returns [`ScanError::LibraryNotFound`] if the library does not exist or
    /// [`ScanError::InvalidRoot`] if the configured root cannot be scanned.
    pub fn scan_library(&self, library_id: i64) -> Result<ScanReport, ScanError> {
        let library = {
            let conn = self.conn();
            library_by_id(&conn, library_id)?.ok_or(ScanError::LibraryNotFound)?
        };

        verify_library_root(&library.path)?;

        let disk_nodes = walk_dir(&library.path)?;

        let mut conn = self.conn();
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut existing_by_path: HashMap<PathBuf, Entry> = entries_for_library(&tx, library_id)?
            .into_iter()
            .map(|entry| (entry.path.clone(), entry))
            .collect();
        let mut ids_by_path: HashMap<PathBuf, i64> = existing_by_path
            .iter()
            .map(|(path, entry)| (path.clone(), entry.id))
            .collect();
        let mut report = ScanReport::default();

        {
            let mut upsert_stmt = tx.prepare(UPSERT_ENTRY_SQL)?;
            for node in disk_nodes {
                let existing_entry = existing_by_path.remove(&node.path);
                let parent_id = parent_id_for_path(&library.path, &node.path, &ids_by_path)?;
                let entry = node.entry_write(library.id, parent_id);
                let id = match existing_entry {
                    Some(existing_entry) if !needs_update(&existing_entry, &entry) => {
                        existing_entry.id
                    }
                    Some(_) => {
                        report.updated += 1;
                        upsert_entry_stmt(&mut upsert_stmt, &entry)?
                    }
                    None => {
                        report.inserted += 1;
                        upsert_entry_stmt(&mut upsert_stmt, &entry)?
                    }
                };
                ids_by_path.insert(node.path, id);
            }
        }

        report.deleted = existing_by_path.len();
        {
            let mut delete_stmt = tx.prepare("DELETE FROM entries WHERE id = ?1")?;
            for entry in existing_by_path.into_values() {
                delete_stmt.execute([entry.id])?;
            }
        }

        tx.commit()?;
        Ok(report)
    }
}

/// Ensures the configured library root exists and is a directory.
fn verify_library_root(root: &Path) -> Result<(), ScanError> {
    let metadata = match fs::metadata(root) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Err(ScanError::InvalidRoot {
                path: root.to_path_buf(),
                reason: "missing",
            });
        }
        Err(err) => return Err(ScanError::Io(err)),
    };

    if !metadata.is_dir() {
        return Err(ScanError::InvalidRoot {
            path: root.to_path_buf(),
            reason: "not a directory",
        });
    }

    Ok(())
}

/// Helper function to check if a file/dir starts with a dot
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

/// Recursively walks `root`, returning all files and directories encountered.
/// Hidden files and directories (dotfiles) are skipped. Symlink targets are not
/// traversed because `WalkDir::new(root)` defaults to `follow_links(false)`.
fn walk_dir(root: &std::path::Path) -> Result<Vec<DiskNode>, ScanError> {
    let mut result = Vec::new();

    let walker = WalkDir::new(root)
        .min_depth(1)
        .into_iter()
        .filter_entry(|entry| entry.depth() == 0 || !is_hidden(entry));

    for entry in walker {
        let entry = entry?;
        let path = entry.path().to_path_buf();
        let is_dir = entry.file_type().is_dir();
        let metadata = entry.metadata()?;

        let size = if is_dir {
            None
        } else {
            Some(
                i64::try_from(metadata.len()).map_err(|_| ScanError::InvalidMetadata {
                    path: path.clone(),
                    reason: "file size does not fit in SQLite integer",
                })?,
            )
        };
        let modified = metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ScanError::InvalidMetadata {
                path: path.clone(),
                reason: "mtime is before Unix epoch",
            })?;
        let mtime = i64::try_from(modified.as_secs()).map_err(|_| ScanError::InvalidMetadata {
            path: path.clone(),
            reason: "mtime does not fit in SQLite integer",
        })?;

        result.push(DiskNode {
            path,
            is_dir,
            size,
            mtime: Some(mtime),
        });
    }

    Ok(result)
}

/// Resolves a scanned path's database parent id from paths already processed in this scan.
fn parent_id_for_path(
    library_root: &Path,
    path: &Path,
    ids_by_path: &HashMap<PathBuf, i64>,
) -> Result<Option<i64>, ScanError> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid_path(path, "has no parent"))?;

    if parent == library_root {
        return Ok(None);
    }

    if !parent.starts_with(library_root) {
        return Err(invalid_path(path, "outside library root"));
    }

    ids_by_path
        .get(parent)
        .copied()
        .map(Some)
        .ok_or_else(|| invalid_path(path, "parent was not scanned before child"))
}

/// Builds a path hierarchy error without repeating `PathBuf` conversion at call sites.
fn invalid_path(path: &Path, reason: &'static str) -> ScanError {
    ScanError::InvalidPath {
        path: path.to_path_buf(),
        reason,
    }
}

/// Returns whether persisted entry fields differ from the freshly scanned state.
fn needs_update(existing: &Entry, scanned: &EntryWrite) -> bool {
    existing.parent_id != scanned.parent_id
        || existing.kind != scanned.kind
        || existing.item_type != scanned.item_type
        || existing.size != scanned.size
        || existing.mtime != scanned.mtime
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs as test_fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::database::Database;
    use crate::database::models::data::LibraryKind;
    use crate::database::services::entry::EntryWrite;

    static NEXT_TEMP_ID: AtomicUsize = AtomicUsize::new(0);

    struct TempRoot {
        path: PathBuf,
    }

    impl TempRoot {
        fn new(name: &str) -> Self {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let mut path = std::env::temp_dir();
            path.push(format!("media-lib-rs-{name}-{}-{id}", std::process::id()));

            let _ = test_fs::remove_dir_all(&path);
            test_fs::create_dir_all(&path).unwrap();

            Self { path }
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = test_fs::remove_dir_all(&self.path);
        }
    }

    fn create_db_with_library(root: &Path) -> (Database, i64) {
        let db = Database::open_in_memory().unwrap();
        let library_id = db
            .libraries()
            .add("Test library", root, LibraryKind::Movies)
            .unwrap();

        (db, library_id)
    }

    fn scan(db: &Database, library_id: i64) -> Result<ScanReport, ScanError> {
        db.scanner().scan_library(library_id)
    }

    fn entry_by_path(entries: &[Entry], path: &Path) -> Entry {
        entries
            .iter()
            .find(|entry| entry.path == path)
            .unwrap_or_else(|| panic!("missing entry for path {path:?}"))
            .clone()
    }

    fn sorted_names(entries: &[Entry]) -> Vec<String> {
        let mut names: Vec<_> = entries
            .iter()
            .map(|entry| entry.name().to_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn first_scan_creates_hierarchy() {
        let root = TempRoot::new("first-scan-creates-hierarchy");
        let movie_dir = root.path.join("Movie");
        test_fs::create_dir_all(&movie_dir).unwrap();
        test_fs::write(movie_dir.join("poster.jpg"), b"poster").unwrap();
        test_fs::write(movie_dir.join("movie.mkv"), b"movie").unwrap();

        let (db, library_id) = create_db_with_library(&root.path);
        let report = scan(&db, library_id).unwrap();

        assert_eq!(report.inserted, 3);

        let libraries = db.libraries();
        let entries = db.entries();
        let titles = libraries.titles(library_id).unwrap();
        assert_eq!(titles.len(), 1);
        assert_eq!(titles[0].name(), "Movie");
        assert_eq!(titles[0].parent_id, None);

        let children = entries.children(titles[0].id).unwrap();
        assert_eq!(sorted_names(&children), ["movie.mkv", "poster.jpg"]);
        assert!(
            children
                .iter()
                .all(|entry| entry.parent_id == Some(titles[0].id))
        );
    }

    #[test]
    fn deep_nesting_preserves_all_parents() {
        let root = TempRoot::new("deep-nesting-preserves-all-parents");
        let dir_a = root.path.join("A");
        let dir_b = dir_a.join("B");
        let dir_c = dir_b.join("C");
        let file = dir_c.join("file.mkv");
        test_fs::create_dir_all(&dir_c).unwrap();
        test_fs::write(&file, b"movie").unwrap();

        let (db, library_id) = create_db_with_library(&root.path);
        scan(&db, library_id).unwrap();

        let all = db.libraries().get_all_entries(library_id).unwrap();
        let a = entry_by_path(&all, &dir_a);
        let b = entry_by_path(&all, &dir_b);
        let c = entry_by_path(&all, &dir_c);
        let file = entry_by_path(&all, &file);

        assert_eq!(a.parent_id, None);
        assert_eq!(b.parent_id, Some(a.id));
        assert_eq!(c.parent_id, Some(b.id));
        assert_eq!(file.parent_id, Some(c.id));
    }

    #[test]
    fn root_level_files_remain_root_entries() {
        let root = TempRoot::new("root-level-files-remain-root-entries");
        let root_file = root.path.join("root_file.mkv");
        test_fs::write(&root_file, b"movie").unwrap();

        let (db, library_id) = create_db_with_library(&root.path);
        scan(&db, library_id).unwrap();

        let titles = db.libraries().titles(library_id).unwrap();
        assert_eq!(titles.len(), 1);
        assert_eq!(titles[0].path, root_file);
        assert_eq!(titles[0].parent_id, None);
    }

    #[test]
    fn scan_hidden_root_keeps_visible_children() {
        let root = TempRoot::new("scan-hidden-root-keeps-visible-children");
        let hidden_root = root.path.join(".hidden-library");
        let visible_file = hidden_root.join("movie.mkv");
        test_fs::create_dir_all(&hidden_root).unwrap();
        test_fs::write(&visible_file, b"movie").unwrap();

        let (db, library_id) = create_db_with_library(&hidden_root);
        let report = scan(&db, library_id).unwrap();

        assert_eq!(report.inserted, 1);
        let entries = db.libraries().get_all_entries(library_id).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, visible_file);
    }

    #[test]
    fn rescan_updates_file_without_changing_parent() {
        let root = TempRoot::new("rescan-updates-file-without-changing-parent");
        let dir = root.path.join("Movie");
        let file = dir.join("movie.mkv");
        test_fs::create_dir_all(&dir).unwrap();
        test_fs::write(&file, b"old").unwrap();

        let (db, library_id) = create_db_with_library(&root.path);
        scan(&db, library_id).unwrap();
        let before = db.libraries().get_all_entries(library_id).unwrap();
        let before_file = entry_by_path(&before, &file);

        test_fs::write(&file, b"new file contents").unwrap();
        let report = scan(&db, library_id).unwrap();
        assert!(report.updated >= 1);

        let after = db.libraries().get_all_entries(library_id).unwrap();
        let after_file = entry_by_path(&after, &file);
        assert_eq!(after_file.id, before_file.id);
        assert_eq!(after_file.parent_id, before_file.parent_id);
        assert_ne!(after_file.size, before_file.size);
    }

    #[test]
    fn rescan_repairs_corrupt_parent_id() {
        let root = TempRoot::new("rescan-repairs-corrupt-parent-id");
        let dir = root.path.join("Movie");
        let file = dir.join("movie.mkv");
        test_fs::create_dir_all(&dir).unwrap();
        test_fs::write(&file, b"movie").unwrap();

        let (db, library_id) = create_db_with_library(&root.path);
        scan(&db, library_id).unwrap();
        let before = db.libraries().get_all_entries(library_id).unwrap();
        let dir_entry = entry_by_path(&before, &dir);
        let file_entry = entry_by_path(&before, &file);

        {
            let scanner = db.scanner();
            let conn = scanner.conn();
            conn.execute(
                "UPDATE entries SET parent_id = NULL WHERE id = ?1",
                [file_entry.id],
            )
            .unwrap();
        }

        let report = scan(&db, library_id).unwrap();
        assert!(report.updated >= 1);
        let after = db.libraries().get_all_entries(library_id).unwrap();
        let repaired_file = entry_by_path(&after, &file);
        assert_eq!(repaired_file.parent_id, Some(dir_entry.id));
    }

    #[test]
    fn rescan_deletes_removed_entries_only_for_scanned_library() {
        let root = TempRoot::new("rescan-deletes-only-scanned-library");
        let other_root = TempRoot::new("rescan-deletes-only-other-library");
        let file = root.path.join("movie.mkv");
        test_fs::write(&file, b"movie").unwrap();

        let db = Database::open_in_memory().unwrap();
        let libraries = db.libraries();
        let entries = db.entries();
        let library_id = libraries
            .add("Library 1", &root.path, LibraryKind::Movies)
            .unwrap();
        let other_library_id = libraries
            .add("Library 2", &other_root.path, LibraryKind::Movies)
            .unwrap();

        scan(&db, library_id).unwrap();
        let duplicate = EntryWrite::scanned(other_library_id, None, &file, false, Some(5), Some(1));
        let duplicate_id = entries.upsert_entry(&duplicate).unwrap();

        test_fs::remove_file(&file).unwrap();
        let report = scan(&db, library_id).unwrap();

        assert_eq!(report.deleted, 1);
        assert!(libraries.get_all_entries(library_id).unwrap().is_empty());
        assert_eq!(entries.get(duplicate_id).unwrap().unwrap().path, file);
    }

    #[test]
    fn rescan_deletes_removed_folder_and_children() {
        let root = TempRoot::new("rescan-deletes-removed-folder-and-children");
        let dir = root.path.join("Movie");
        let file = dir.join("movie.mkv");
        test_fs::create_dir_all(&dir).unwrap();
        test_fs::write(&file, b"movie").unwrap();

        let (db, library_id) = create_db_with_library(&root.path);
        scan(&db, library_id).unwrap();

        test_fs::remove_dir_all(&dir).unwrap();
        let report = scan(&db, library_id).unwrap();

        assert_eq!(report.deleted, 2);
        assert!(
            db.libraries()
                .get_all_entries(library_id)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn scan_rejects_regular_file_root_without_deleting_existing_entries() {
        let root = TempRoot::new("scan-rejects-file-root");
        let file_root = root.path.join("not-a-directory");
        test_fs::write(&file_root, b"not a directory").unwrap();

        let db = Database::open_in_memory().unwrap();
        let library_id = db
            .libraries()
            .add("File root", &file_root, LibraryKind::Movies)
            .unwrap();
        let old_path = root.path.join("old.mkv");
        let old_entry = EntryWrite::scanned(library_id, None, &old_path, false, Some(3), Some(1));
        let old_entry_id = db.entries().upsert_entry(&old_entry).unwrap();

        let err = scan(&db, library_id).unwrap_err();

        assert!(matches!(err, ScanError::InvalidRoot { .. }));
        assert!(db.entries().get(old_entry_id).unwrap().is_some());
    }

    #[test]
    fn failed_scan_write_rolls_back_all_changes() {
        let root = TempRoot::new("failed-scan-write-rolls-back-all-changes");
        let old_file = root.path.join("old.mkv");
        let new_file = root.path.join("new.mkv");
        test_fs::write(&old_file, b"old").unwrap();

        let (db, library_id) = create_db_with_library(&root.path);
        scan(&db, library_id).unwrap();
        let old_entry = entry_by_path(
            &db.libraries().get_all_entries(library_id).unwrap(),
            &old_file,
        );

        test_fs::remove_file(&old_file).unwrap();
        test_fs::write(&new_file, b"new").unwrap();

        let scanner = db.scanner();
        let escaped_new_path = new_file.to_str().unwrap().replace('\'', "''");
        let trigger_sql = format!(
            "CREATE TRIGGER fail_new_entry_insert \
             BEFORE INSERT ON entries \
             WHEN NEW.path = '{escaped_new_path}' \
             BEGIN \
                 SELECT RAISE(ABORT, 'forced scan failure'); \
             END;"
        );
        {
            let conn = scanner.conn();
            conn.execute_batch(&trigger_sql).unwrap();
        }

        let err = scanner.scan_library(library_id).unwrap_err();
        assert!(matches!(err, ScanError::Db(_)));

        let after = db.libraries().get_all_entries(library_id).unwrap();
        assert!(after.iter().any(|entry| entry.id == old_entry.id));
        assert!(after.iter().all(|entry| entry.path != new_file));
    }
}
