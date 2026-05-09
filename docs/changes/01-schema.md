# Core schema: libraries and entries

The whole database revolves around two tables. The original design used a fixed four-level hierarchy (`libraries -> series -> parts -> items`), which forced every piece of media into a shape that does not match the filesystem. The new design replaces those four tables with a single recursive `entries` table.

## Why one recursive table

The old hierarchy assumed every item lives under a series, which lives under a library, with parts in between. This works for a manga collection where each series has volumes and each volume has chapters. It breaks for almost everything else. Movies often sit one level deep with no series at all. A standalone audiobook is a single file. Music can be artist/album/track or just artist/track for indie releases. Photos can be nested arbitrarily deep.

A self-referencing `entries` table mirrors the filesystem exactly. Any depth, any shape. The scanner walks directories and inserts a row per node. No special cases per media type.

## Schema

```sql
CREATE TABLE libraries (
    id    INTEGER PRIMARY KEY,
    name  TEXT NOT NULL,
    path  TEXT NOT NULL UNIQUE,
    kind  TEXT NOT NULL
        CHECK (kind IN ('movies','tv','manga','books','music','audiobooks'))
);

CREATE TABLE entries (
    id          INTEGER PRIMARY KEY,
    library_id  INTEGER NOT NULL REFERENCES libraries(id) ON DELETE CASCADE,
    parent_id   INTEGER REFERENCES entries(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    path        TEXT NOT NULL,
    kind        TEXT NOT NULL
        CHECK (kind IN ('folder','file')),
    item_type   TEXT
        CHECK (item_type IN ('img','vid','aud','read')),
    size        INTEGER,
    mtime       INTEGER,
    UNIQUE(library_id, path)
);

CREATE INDEX entries_parent  ON entries(parent_id);
CREATE INDEX entries_library ON entries(library_id);
```

## Column notes

`libraries.kind` is stored as text rather than an integer so a glance at the table tells you what each library is. The `CHECK` constraint pins it to the seven legal values, giving us enum-like safety without losing readability — a typo in the Rust `to_str`/`from_str` mapping or a future migration that forgets a value fails at insert time instead of silently corrupting the table.

`entries.parent_id` is `NULL` for top-level entries. A top-level entry is a Title, the thing the user rates and favorites. Anything below it is a descendant.

`entries.path` is the absolute path on disk. Storing absolute means a library moved to a different drive needs a path rewrite, but it also means looking up an entry from a filesystem event is a direct index probe. `UNIQUE(library_id, path)` prevents the same file being indexed twice in the same library and is the natural key the scanner upserts against.

`entries.kind` is constrained to `folder` or `file`. Splitting on this is more useful than splitting on `parent_id IS NULL`, because a Title can be either (a single-file movie at the library root is a Title and a file).

`entries.item_type` is only set for files; folders leave it `NULL`. The constraint matches the existing `ItemType` enum on the Rust side. The "files have a type, folders don't" rule is enforced in code rather than via a multi-column `CHECK`, leaving room for future entry kinds without a migration.

`entries.size` and `entries.mtime` are only set for files. They power the cheap change detection in the scanner: if neither has changed since the last scan, the file is skipped.

## How the user-facing concepts map

| User says | Database row                                      |
|-----------|---------------------------------------------------|
| Title     | `entry` with `parent_id IS NULL`                  |
| Series    | folder-entry, usually a Title                     |
| Volume    | folder-entry, child of a Title                    |
| Season    | folder-entry, child of a show Title               |
| Album     | folder-entry, child of an artist Title            |
| Chapter   | folder-entry (page-based) or file-entry (cbz/cbr) |
| Episode   | file-entry under a season folder                  |
| Track     | file-entry under an album folder                  |
| Page      | file-entry with `item_type='img'` under a chapter |

The scanner does not care about these labels. The labels exist in the UI layer.

## Edge cases the design handles for free

- Standalone movie at library root: one row, `kind='file'`, `parent_id=NULL`.
- Avengers folder with two movie files: one folder-row (the Title) plus two file-rows under it.
- Manga without volumes: Title-folder, chapter-folders directly under it, page-files under each chapter. No volume layer required because none exists.
- Indie song with no album: Title-folder for the artist, file-row directly under it.
The shape of the tree comes from the disk. The DB does not impose one.

## Indexes

`entries_parent` makes "get the children of this folder" a simple lookup. This is the dominant browse query.

`entries_library` makes per-library listings cheap and is needed for the scan diff (load all entries for a library at once).

A composite `(library_id, parent_id)` index might be worth adding later if profiling shows the per-library top-level Title listing is slow. Skipping it for now since the two single-column indexes already cover most of the access patterns and the data set is small.
