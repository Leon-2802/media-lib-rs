# Roadmap

## Phase 1 — Scanner

**Goal:** Discover and track every file/folder in a user's local media library so the DB always reflects what is on disk.

- [ ] `walk_dir()` — Recursively walk a library root directory. Skip hidden files, dotfiles, and symlinks. Return a flat list of `DiskNode`.
- [ ] `diff()` — Compare on-disk state against existing DB entries. Produce three lists: `to_insert`, `to_update`, `to_delete`.
- [ ] `scan_library()` — Fetch library root → walk → diff → apply changes in a single `BEGIN IMMEDIATE` transaction. Return a `ScanReport`.
- [ ] Re-scan detection: detect removed files (gone from disk, still in DB) and deleted folders.
- [ ] CLI commands: `add-library <name> <path> <type>`, `scan <library-id>`, `list-libraries`, `remove-library <id>`, `list-entries <library-id>`

---

## Phase 2 — Web API

**Goal:** Expose all library data over HTTP so any client (WebUI, mobile app, scripts) can read and mutate state.

- [ ] Add an async web framework (Axum or Actix).
- [ ] REST endpoints:
  - `GET /libraries` — list all libraries
  - `POST /libraries` — add a library
  - `DELETE /libraries/:id` — remove a library
  - `POST /libraries/:id/scan` — trigger a rescan
  - `GET /libraries/:id/entries` — top-level entries (`?parent_id=N` for subtree)
  - `GET /entries/:id` — single entry details
  - `GET /entries/:id/children` — children of an entry
  - `GET /entries/:id/ancestors` — path from root to entry
  - `POST /entries/:id/rating` — set rating (1-10)
  - `DELETE /entries/:id/rating` — clear rating
  - `POST /entries/:id/favorite` — add to favorites
  - `DELETE /entries/:id/favorite` — remove from favorites
  - `POST /entries/:id/watched` — mark as watched/unwatched
  - `POST /entries/:id/tags` — add tag(s) `{"tags": ["anime", "favorite"]}`
  - `DELETE /entries/:id/tags/:tag` — remove a tag
  - `GET /tags` — all tags
  - `GET /entries/:id/tags` — tags for an entry
  - `GET /search?q=` — search entries by name (fuzzy)
  - `GET /search-history` — recent searches
- [ ] Serve over a local TCP port or Unix socket (for Pi deployment behind a reverse proxy).

---

## Phase 3 — Metadata & Ratings Sync

**Goal:** Enrich raw file entries with rich metadata — posters, synopses, year, genres — and push user ratings and watched status to TMDB, TVDb, AniList, and MyAnimeList so they show up across all the user's accounts on those platforms.

- [ ] **Migration:** Add `metadata` table to track enrichment state per entry.
  ```sql
  CREATE TABLE metadata (
      entry_id     INTEGER PRIMARY KEY REFERENCES entries(id) ON DELETE CASCADE,
      source       TEXT NOT NULL CHECK (source IN ('tmdb','tvdb','anilist','mal')),
      external_id  TEXT NOT NULL,
      title        TEXT,
      year         INTEGER,
      synopsis     TEXT,
      poster_url   TEXT,
      backdrop_url TEXT,
      genres       TEXT,
      fetched_at   INTEGER NOT NULL,
      UNIQUE(entry_id, source)
  );
  ```
- [ ] `MetadataService` with `lookup(entry_id, source)` — normalize entry name, call the appropriate API (TMDB for movies/TV, AniList/MAL for anime/manga), pick best match via fuzzy scoring.
- [ ] `SyncService` — push ratings and watched state to TMDB/TVDb/AniList/MAL:
    - `sync_rating(entry_id, source, rating)` — post user's 1-10 rating to their TMDB/TVDb/AniList account
    - `sync_watched(entry_id, source, watched)` — mark item as watched/unwatched on the platform
    - Use OAuth tokens stored in config to authenticate against each platform
- [ ] Auto-fetch: after a scan completes, queue new/unmatched entries for background metadata enrichment.
- [ ] Manual refresh: `POST /entries/:id/refresh-metadata?source=anilist` to re-fetch.
- [ ] Cache fetched metadata locally to avoid repeated API calls.
- [ ] Support multiple sources per entry (e.g. TMDB *and* AniList for a cross-media item).

---

## Phase 4 — WebUI

**Goal:** Provide a polished, responsive browser interface for browsing, searching, rating, tagging, and managing the media library — accessible from desktop and mobile.

- [ ] SPA frontend (React, Leptos, or Next.js).
- [ ] Library browser: grid/list toggle, navigate folder hierarchy, breadcrumb trail.
- [ ] Filter/sort: by name, rating, tag, item type, library, watched status.
- [ ] Entry detail view: poster, backdrop, synopsis, year, genres, rating widget, watched toggle, tag list, file info (size, path, mtime).
- [ ] Favorites page: dedicated view of all favorited entries.
- [ ] Watched/Unwatched filter: view only unwatched items in a library.
- [ ] Search UI: live search-as-you-type with search history dropdown.
- [ ] Tag management: create tags, assign/remove tags from entries inline.
- [ ] Dark/light theme with system preference detection.
- [ ] Mobile-friendly layout (touch-friendly grid, swipe gestures for rating).
- [ ] Settings page: configure library paths, TMDB/TVDb/AniList/MAL API keys and OAuth tokens.
