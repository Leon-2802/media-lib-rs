# Migrations and future work

## Migrations from day one

The `rusqlite_migration` crate handles versioned schema changes. Wiring it up before there is any data is free; wiring it up later is a manual fixup script per existing user.

```rust
// database/migrations.rs
use rusqlite_migration::{Migrations, M};

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(include_str!("sql/0001_initial.sql")),
    ])
}
```

`Database::open` runs:

1. `PRAGMA journal_mode=WAL`
2. `PRAGMA foreign_keys=ON`
3. `migrations().to_latest(&mut conn)`

The first migration creates every table from the schema and rating-layer documents in one file. Future schema changes get their own file (`0002_add_hash.sql`, etc.) and a new `M::up(...)` line.

## File layout

```
src/database/
  mod.rs                   Database struct, open(), service factories
  migrations.rs            rusqlite_migration setup
  sql/
    0001_initial.sql       libraries, entries, tags, ratings, favorites, search_history
  models/
    data.rs                Library, Entry, LibraryKind, EntryKind, ItemType
    app.rs                 Tag, Rating, Favorite
  services/
    library.rs
    entry.rs
    tag.rs
    rating.rs
    favorite.rs
    scan.rs
```

Mostly the same shape the project already has, just with more services and a `sql/` directory for migration files.

## What changes in the existing code

- `Library` loses its `series: Vec<Series>` field. Callers query `EntryService::children(title_id)` instead.
- `Series`, `Part`, and the old `Item` structs go away entirely. `Entry` replaces them.
- The four `*Service` files keep their structure (one file per service, each owning an `Arc<Mutex<Connection>>`). The methods change.
- The existing `LibService` tests keep their shape: open in-memory DB, exercise CRUD, assert. Just renamed and updated for the new model.
- `ItemType::from_path` stays. Adding `to_str` and `from_str` for TEXT serialization in the DB.

`i32` becomes `i64` everywhere ids appear.

## Future work backlog

Roughly in the order they would matter:

**Per-kind metadata.** Right now an entry has a name from the filesystem and that is it. Adding a separate `entry_metadata(entry_id, source, json)` table lets a future enrichment pass attach things like clean titles, year, episode numbers, cover art URLs, external IDs (TMDB, AniList, Goodreads). Keeping it in JSON keeps the schema flexible without locking in any one external service.

**Cover images and thumbnails.** Either a `thumbnails(entry_id, kind, path)` table pointing at sidecar files in a cache directory, or storing small blobs inline. Cache directory scales better.

**Full-text search.** SQLite's FTS5 virtual tables work well for entry name search. Add when the linear `LIKE` scan starts feeling slow.

**Watch and read progress.** A `playback(entry_id, position, finished_at)` table to remember where the user left off in a video, audiobook, or book.

**Hash-based change detection.** Add a `hash TEXT` column on `entries` and a scanner mode that hashes new files. Lets renames preserve ratings, tags, and favorites by matching on bytes.

**File watching.** Live updates via the `notify` crate, feeding into `EntryService` upserts. Useful once a GUI exists.

**Multi-user.** Adds `user_id` columns to `ratings`, `favorites`, `search_history`, `playback`. Single-user is fine for now.

## What is explicitly out of scope for v1

- External API integrations (TMDB, AniList, Goodreads).
- Transcoding, streaming, playback. The library indexes media; it does not play it.
- Network sync between machines.
- A web UI.

The CLI is the only user surface for the first cut.
