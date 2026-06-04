CREATE TABLE libraries (
    id    INTEGER PRIMARY KEY,
    name  TEXT NOT NULL,
    path  TEXT NOT NULL UNIQUE,
    kind  TEXT NOT NULL
        CHECK (kind IN ('movies','tv','manga','books','audio'))
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

CREATE INDEX entries_parent    ON entries(parent_id);
CREATE INDEX entries_library   ON entries(library_id);
CREATE INDEX entries_size_mtime ON entries(size, mtime);

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
