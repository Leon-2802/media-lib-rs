//! # Database Services
//!
//! Stateless service structs that provide business-logic operations on top of
//! the raw SQLite connection. Each service is self-contained and holds a cloned
//! `Arc<Mutex<Connection>>` so it can be freely shared.
//!
//! ## Services
//!
//! - [`entry::EntryService`] — query and manipulate file/folder entries
//! - [`library::LibraryService`] — manage library records
//! - [`tag::TagService`] — create/attach/detach tags
//! - [`rating::RatingService`] — per-entry ratings
//! - [`favorite::FavoriteService`] — favorited entries
//! - [`search_history::SearchHistoryService`] — search query history
//! - [`scan::ScanService`] — directory scanning and DB sync

pub mod entry;
pub mod favorite;
pub mod library;
pub mod rating;
pub mod scan;
pub mod search_history;
pub mod tag;
