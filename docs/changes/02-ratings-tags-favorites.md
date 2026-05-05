# Ratings, tags, and favorites

These three features all attach to entries. The schema is uniform regardless of where in the tree the rated/tagged/favorited thing lives. A Title gets a rating the same way a single file does.

## Schema

```sql
CREATE TABLE tags (
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE
);

CREATE TABLE entry_tags (
    entry_id INTEGER NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    tag_id   INTEGER NOT NULL REFERENCES tags(id)    ON DELETE CASCADE,
    PRIMARY KEY (entry_id, tag_id)
);

CREATE TABLE ratings (
    entry_id INTEGER PRIMARY KEY REFERENCES entries(id) ON DELETE CASCADE,
    rating   INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 10)
);

CREATE TABLE favorites (
    entry_id INTEGER PRIMARY KEY REFERENCES entries(id) ON DELETE CASCADE,
    added_at INTEGER NOT NULL
);

CREATE TABLE search_history (
    id    INTEGER PRIMARY KEY,
    query TEXT NOT NULL,
    at    INTEGER NOT NULL
);
```

## Why one set of tables instead of per-level

The original design had `SeriesRating` and `PartRating` as separate models. Adding ratings to anything else (an episode, a single chapter, an album) would have meant another table per level. With a single `ratings(entry_id, rating)` table the same query handles every case.

Cost: the schema cannot enforce "only Titles can be rated" or "manga pages cannot be favorited". That rule lives in the application layer.

## The cutoff lives in code, not schema

The design conversation landed on a rule: rate Titles, rate folders inside them (volumes, seasons, albums), rate file-entries that are themselves a complete consumable (a movie, an episode, a song, a book file). Do not rate things that are too granular: manga pages.

In the schema this is not a constraint. The DB allows any entry to be rated. The decision happens in the service or UI layer:

```rust
fn is_rateable(entry: &Entry) -> bool {
    match entry.kind {
        EntryKind::Folder => true,
        EntryKind::File   => !matches!(entry.item_type, Some(ItemType::Img)),
    }
}
```

Folders are always fair game (any folder is a meaningful grouping the user might want to rate). Image files are not (they are pages). All other file types are. If the rule changes later, it is a one-function change with no migration.

## Tag table notes

`name TEXT UNIQUE COLLATE NOCASE` means tags are case-insensitive. Adding "Fantasy" when "fantasy" already exists fails the unique constraint and the service can reuse the existing row. Display casing follows whoever inserted first, which is good enough for v1.

`entry_tags` is a plain join table. Many entries can share a tag, an entry can have many tags. Cascading on both sides means deleting an entry cleans up its tag links, deleting a tag cleans up everywhere it was attached.

## Cascade behavior

Every foreign key uses `ON DELETE CASCADE`. The intent: when a file is removed during a scan, all its application-layer attachments go with it. The DB takes care of cleanup so the scanner does not have to.

The `tags` table itself is the exception. Tag definitions outlive entry deletions. Removing the last entry that used a tag does not delete the tag. This is intentional: the user might tag the same media again later and expect to reuse the existing tag.

## Renames lose attachments

The cheap change-detection mode (path + size + mtime) treats a rename as a delete followed by an insert. So a renamed file loses its rating, favorite, and tags. The lost-attachment risk is real but usually minor for typical reorganization. The escape hatch is the future hash-based sync mode. Schema-wise it would only need a `hash TEXT` column on `entries`. No table changes.

## Search history

`search_history` is the simplest possible thing. Every search query the user runs gets a row with a timestamp. It exists so the UI can suggest recent searches and so the user can review what they have looked up.

Out of scope for v1: per-user history (single-user app), result counts, click-throughs. If retention becomes an issue, a periodic prune query (`DELETE FROM search_history WHERE at < ?`) is enough.
