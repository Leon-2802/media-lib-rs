//! # Database Migrations
//!
//! Schema migrations for the SQLite database, managed via `rusqlite_migration`.
//! Each migration is defined as an ordered step; `to_latest` applies any pending
//! steps based on the current `PRAGMA user_version`.

// Each `M::up` is one ordered step keyed by position in the Vec.
// `to_latest` runs the tail missing from `PRAGMA user_version`.
// Append-only: new change = new `sql/` file + new `M::up` line.

use rusqlite_migration::{M, Migrations};

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(include_str!("sql/0001_initial.sql"))])
}
