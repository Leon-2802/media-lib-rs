# Rust models and services

## Models

The new models are flat. The old `Library { series: Vec<Series> }` shape forced every read to either return an empty vec or eagerly hydrate the entire tree. The replacement returns rows. Trees are recovered on demand by querying for children.

```rust
// models/data.rs
pub enum LibraryKind {
    Movies,
    Tv,
    Manga,
    Books,
    Music,
    Audiobooks,
}

pub enum EntryKind {
    Folder,
    File,
}

pub enum ItemType {
    Img,
    Vid,
    Aud,
    Read,
}

pub struct Library {
    pub id:   i64,
    pub name: String,
    pub path: PathBuf,
    pub kind: LibraryKind,
}

pub struct Entry {
    pub id:         i64,
    pub library_id: i64,
    pub parent_id:  Option<i64>,
    pub name:       String,
    pub path:       PathBuf,
    pub kind:       EntryKind,
    pub item_type:  Option<ItemType>,
    pub size:       Option<i64>,
    pub mtime:      Option<i64>,
}

// models/app.rs
pub struct Tag      { pub id: i64, pub name: String }
pub struct Rating   { pub entry_id: i64, pub rating: i8 }
pub struct Favorite { pub entry_id: i64, pub added_at: i64 }
```

`i32` becomes `i64`. SQLite's ROWID is 64-bit and `last_insert_rowid()` returns `i64`. Storing it as `i32` was a latent bug.

`Series`, `Part`, and the old `Item` struct go away. `Entry` covers all three.

`PathBuf` for paths instead of `String` so callers do not have to keep re-parsing.

`ItemType::from_path(&Path) -> Option<Self>` from the existing code stays as is, plus matching `as_str` and `from_str` helpers for storing in TEXT columns.

## Services

The connection-sharing pattern from the original design (`Arc<Mutex<Connection>>` cloned into each service) carries over without changes. The factory methods on `Database` just expand:

```rust
pub fn libraries(&self) -> LibraryService
pub fn entries(&self)   -> EntryService
pub fn tags(&self)      -> TagService
pub fn ratings(&self)   -> RatingService
pub fn favorites(&self) -> FavoriteService
pub fn scanner(&self)   -> ScanService
```

### LibraryService

CRUD on libraries plus a convenience method for listing the top-level entries (the Titles) of a library.

```rust
fn add(&self, name: &str, path: &Path, kind: LibraryKind) -> Result<i64>
fn get(&self, id: i64) -> Result<Option<Library>>
fn all(&self) -> Result<Vec<Library>>
fn delete(&self, id: i64) -> Result<bool>
fn titles(&self, library_id: i64) -> Result<Vec<Entry>>
```

`titles` is `SELECT * FROM entries WHERE library_id = ?1 AND parent_id IS NULL`.

### EntryService

The workhorse. Tree queries plus file metadata upserts.

```rust
fn get(&self, id: i64) -> Result<Option<Entry>>
fn by_path(&self, library_id: i64, path: &Path) -> Result<Option<Entry>>
fn children(&self, parent_id: i64) -> Result<Vec<Entry>>
fn descendants(&self, root_id: i64) -> Result<Vec<Entry>>   // recursive CTE
fn ancestors(&self, entry_id: i64) -> Result<Vec<Entry>>     // recursive CTE
fn title_of(&self, entry_id: i64) -> Result<Entry>           // walks up to parent_id IS NULL
fn upsert_file(&self, ...) -> Result<i64>                    // used by scanner
fn upsert_folder(&self, ...) -> Result<i64>                  // used by scanner
fn delete(&self, id: i64) -> Result<bool>
```

`descendants` and `ancestors` use SQLite recursive CTEs. The query for descendants looks like this:

```sql
WITH RECURSIVE tree(id) AS (
    SELECT id FROM entries WHERE id = ?1
    UNION ALL
    SELECT e.id FROM entries e JOIN tree t ON e.parent_id = t.id
)
SELECT * FROM entries WHERE id IN tree;
```

### TagService, RatingService, FavoriteService

Thin wrappers over their tables. No exotic logic.

```rust
// TagService
fn create(&self, name: &str) -> Result<i64>
fn get_or_create(&self, name: &str) -> Result<i64>
fn attach(&self, entry_id: i64, tag_id: i64) -> Result<()>
fn detach(&self, entry_id: i64, tag_id: i64) -> Result<()>
fn for_entry(&self, entry_id: i64) -> Result<Vec<Tag>>
fn entries_with_tag(&self, tag_id: i64) -> Result<Vec<Entry>>

// RatingService
fn set(&self, entry_id: i64, rating: i8) -> Result<()>   // INSERT OR REPLACE
fn get(&self, entry_id: i64) -> Result<Option<i8>>
fn clear(&self, entry_id: i64) -> Result<bool>

// FavoriteService
fn add(&self, entry_id: i64) -> Result<()>
fn remove(&self, entry_id: i64) -> Result<bool>
fn is_favorite(&self, entry_id: i64) -> Result<bool>
fn all(&self) -> Result<Vec<Entry>>     // join through favorites
```

### ScanService

Covered in its own document.

## Locking pattern unchanged

Each service holds an `Arc<Mutex<Connection>>`. Methods acquire the lock at the start and drop it as the function returns. No long-held locks across multiple operations. The mutex is a coarse single-writer guard, which suits SQLite's own threading model.

When the app eventually goes async or GUI, swapping the internals to a connection pool or to `tokio_rusqlite` is a localized change inside `Database`. Service signatures stay the same.
