//! Application-level models for tags, ratings, favorites, and search history.
//!
//! These structs represent data that is associated with entries but kept in
//! separate tables.

/// A tag that can be attached to any entry for categorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    /// Unique numeric identifier.
    pub id: i64,
    /// Display name of the tag. Unique constraint enforced at the DB level.
    pub name: String,
}

/// A rating value assigned to a single entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rating {
    /// The entry this rating belongs to.
    pub entry_id: i64,
    /// The rating value (typically 1–5).
    pub rating: i64,
}

/// Records that an entry has been marked as a favorite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Favorite {
    /// The favorited entry.
    pub entry_id: i64,
    /// Unix timestamp (seconds) when the entry was favorited.
    pub added_at: i64,
}

/// A single search query that was executed by the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHistory {
    /// Unique numeric identifier.
    pub id: i64,
    /// The search query string.
    pub query: String,
    /// Unix timestamp (seconds) when the query was issued.
    pub at: i64,
}
