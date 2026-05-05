# Scan and sync

The scanner is the only piece with non-trivial algorithmic work. It walks a library's filesystem, diffs the result against the database, and applies inserts, updates, and deletes in one transaction.

## High-level flow

```rust
pub fn scan_library(&self, library_id: i64) -> Result<ScanReport> {
    let library = self.libraries.get(library_id)?
        .ok_or(ScanError::LibraryNotFound)?;

    let existing: HashMap<PathBuf, EntryRow> = self.load_existing(library_id)?;
    let on_disk: Vec<DiskNode> = walk_dir(&library.path)?;

    let (to_insert, to_update, to_delete) = diff(&existing, &on_disk);

    let tx = self.conn.lock().unwrap().transaction()?;
    apply_deletes(&tx, &to_delete)?;
    apply_inserts(&tx, &to_insert)?;   // parents before children
    apply_updates(&tx, &to_update)?;
    tx.commit()?;

    Ok(ScanReport { inserted, updated, deleted, skipped })
}
```

## Diff algorithm

Three sets after walking the disk and loading the DB state:

```
to_insert: on_disk paths not present in existing
to_update: on_disk paths present in existing where size or mtime changed
to_delete: existing paths not present in on_disk
```

Files where size and mtime both match are skipped. This is the cheap change-detection mode the design settled on.

```rust
for node in on_disk {
    match existing.get(&node.path) {
        None => to_insert.push(node),
        Some(row) if row.size != node.size || row.mtime != node.mtime => {
            to_update.push(node);
        }
        Some(_) => {} // unchanged
    }
}
let to_delete: Vec<_> = existing.keys()
    .filter(|p| !on_disk_paths.contains(*p))
    .cloned()
    .collect();
```

## Insert order matters

Children reference their parent's row id. Inserts have to be ordered by depth ascending, root first. Easiest way: sort `to_insert` by `path.components().count()` ascending. For each node:

1. Look up `parent_id` by querying for the parent path.
2. Insert the new row.
3. Cache its id keyed by path so subsequent siblings and children find it without another query.

For top-level entries, `parent_id` is `NULL`.

## Deletes go first, cascade does the work

`to_delete` runs before inserts. `ON DELETE CASCADE` on `entries.parent_id` removes all descendants of a deleted folder in one statement. Ratings, tags, and favorites attached to deleted entries also cascade away.

This means a renamed folder (which looks like delete + insert) wipes the whole subtree's attachments. Documented in the rating layer. Future hash-based sync would mitigate.

## Transaction semantics

The whole scan runs in one `BEGIN`. Rationale:

- A scan that crashes halfway leaves the DB consistent (no orphaned children, no half-deleted ratings).
- A reader querying mid-scan sees the old state until commit, never a half-applied state.
- One commit instead of N is much faster on SQLite.

The downside: a very large library (millions of files) holds a write lock for the duration. Acceptable for a personal media collection. If a deployment ever has scan times in the minutes, switching to chunked scans (commit every 10k operations) is a localized change.

## Concurrent scan guard

Two scans of the same library at the same time would race. The simplest guard is `BEGIN IMMEDIATE`, which acquires the write lock at transaction start and fails fast if another writer holds it. The scanner returns `ScanError::Busy` to the caller, which can retry or surface to the user.

A second scan against a different library on the same DB still serializes (one writer at a time in SQLite) but that is acceptable.

## Edge cases handled

- **Symlinks**: skipped entirely. The walker does not descend into them and does not record them as entries. Avoids cycle detection and the broken-symlink class of errors. If linked content needs to be indexed later, the user can point a library at the real location.
- **Hidden files**: skip by name pattern. Defaults: `.DS_Store`, `Thumbs.db`, `desktop.ini`, anything starting with `.` (configurable later).
- **Unreadable files** (permission denied): log, skip the file, continue. Do not abort the whole scan over one bad file.
- **Library root missing**: return `ScanError::RootMissing` immediately. Do not interpret an empty walk as "all files were deleted", because that would wipe the library on a temporarily unmounted drive.

## Edge cases punted

- **Hash-based rename detection**: future opt-in mode. Adds a `hash` column on `entries` and a step that hashes new files and compares against deleted ones to recover their rowid (preserving ratings, tags, favorites).
- **Live file watching**: outside the scanner's scope. The `notify` crate could feed individual upserts into `EntryService` later. Not needed for a CLI-driven workflow.
- **Per-kind scanning rules**: parsing `Show (2019)/S02E07 - Title.mkv` for nicer display names is a separate enrichment step, run after the structural scan. Will hang off a future `entry_metadata` table.

## ScanReport

What the caller gets back:

```rust
pub struct ScanReport {
    pub inserted: usize,
    pub updated:  usize,
    pub deleted:  usize,
    pub skipped:  usize,
    pub errors:   Vec<ScanWarning>,  // unreadable files
}
```

Enough for a CLI summary line ("scanned 1234 files, 12 new, 3 updated, 0 removed") and for tests to assert on.
