//! Data models for the core library and entry hierarchy.
//!
//! Contains the primary domain types: [`Library`], [`Entry`], and the type-classifying
//! enums [`LibraryKind`], [`EntryKind`], and [`ItemType`].

use std::path::{Path, PathBuf};

/// The kind/category of a media library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryKind {
    /// A collection of movies.
    #[default]
    Movies,
    /// A collection of TV series episodes.
    Tv,
    /// A collection of manga/comics.
    Manga,
    /// A collection of ebooks or printed documents.
    Books,
    /// A collection of audio content (music tracks/albums or audiobooks).
    Audio,
}

impl LibraryKind {
    /// Returns the database string representation (e.g. `"movies"`, `"tv"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Movies => "movies",
            Self::Tv => "tv",
            Self::Manga => "manga",
            Self::Books => "books",
            Self::Audio => "audio",
        }
    }

    /// Parses a [`LibraryKind`] from a string slice, returning `None` for unknown values.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "movies" => Some(Self::Movies),
            "tv" => Some(Self::Tv),
            "manga" => Some(Self::Manga),
            "books" => Some(Self::Books),
            "audio" => Some(Self::Audio),
            _ => None,
        }
    }

    /// Parses a [`LibraryKind`] from an owned string, returning `None` for unknown values.
    pub fn from(s: String) -> Option<Self> {
        match s.as_str() {
            "movies" => Some(Self::Movies),
            "tv" => Some(Self::Tv),
            "manga" => Some(Self::Manga),
            "books" => Some(Self::Books),
            "audio" => Some(Self::Audio),
            _ => None,
        }
    }
}

/// Whether an entry in the database is a folder (directory) or a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntryKind {
    /// A directory/folder entry.
    #[default]
    Folder,
    /// A file entry.
    File,
}

impl EntryKind {
    /// Returns `"folder"` or `"file"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Folder => "folder",
            Self::File => "file",
        }
    }

    /// Parses an [`EntryKind`] from a string slice.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "folder" => Some(Self::Folder),
            "file" => Some(Self::File),
            _ => None,
        }
    }

    /// Parses an [`EntryKind`] from an owned string.
    pub fn from(s: String) -> Option<Self> {
        match s.as_str() {
            "folder" => Some(Self::Folder),
            "file" => Some(Self::File),
            _ => None,
        }
    }
}

/// Classifies the type of a file by its extension. Used to determine which
/// player or viewer to launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItemType {
    /// Image file (jpg, png, gif, webp, avif).
    #[default]
    Img,
    /// Video file (mp4, mkv, mov, webm, avi, m4v).
    Vid,
    /// Audio file (mp3, flac, wav, ogg, aac, m4a, opus).
    Aud,
    /// Readable document (pdf, epub).
    Read,
}

impl ItemType {
    /// Infers the [`ItemType`] from a file path's extension. Returns `None` if the
    /// extension is not recognized.
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "avif" => Some(Self::Img),
            "mp4" | "mkv" | "mov" | "webm" | "avi" | "m4v" => Some(Self::Vid),
            "mp3" | "flac" | "wav" | "ogg" | "aac" | "m4a" | "opus" => Some(Self::Aud),
            "pdf" | "epub" => Some(Self::Read),
            _ => None,
        }
    }

    /// Returns `"img"`, `"vid"`, `"aud"`, or `"read"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Img => "img",
            Self::Vid => "vid",
            Self::Aud => "aud",
            Self::Read => "read",
        }
    }

    /// Parses an [`ItemType`] from a string slice.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "img" => Some(Self::Img),
            "vid" => Some(Self::Vid),
            "aud" => Some(Self::Aud),
            "read" => Some(Self::Read),
            _ => None,
        }
    }

    /// Parses an [`ItemType`] from an owned string.
    pub fn from(s: String) -> Option<Self> {
        match s.as_str() {
            "img" => Some(Self::Img),
            "vid" => Some(Self::Vid),
            "aud" => Some(Self::Aud),
            "read" => Some(Self::Read),
            _ => None,
        }
    }
}

/// A registered media library. Libraries are the top-level container for all
/// scanned entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    /// Unique numeric identifier.
    pub id: i64,
    /// Human-readable name (e.g. "Movies", "My Manga").
    pub name: String,
    /// Absolute path to the root of this library on disk.
    pub path: PathBuf,
    /// The kind/category of this library.
    pub kind: LibraryKind,
}

/// A single file or folder entry within a library. Entries form a tree structure
/// via the `parent_id` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Unique numeric identifier.
    pub id: i64,
    /// The library this entry belongs to.
    pub library_id: i64,
    /// `None` for root-level entries; `Some(i64)` for nested entries.
    pub parent_id: Option<i64>,
    /// File or folder name.
    pub name: String,
    /// Absolute path to this entry on disk.
    pub path: PathBuf,
    /// Whether this is a folder or a file.
    pub kind: EntryKind,
    /// Classified media type (image, video, audio, document). `None` for folders.
    pub item_type: Option<ItemType>,
    /// File size in bytes. `None` for folders.
    pub size: Option<i64>,
    /// Last-modified time as a Unix timestamp (seconds). `None` for folders.
    pub mtime: Option<i64>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_img_extensions() {
        for ext in ["jpg", "jpeg", "png", "gif", "webp", "avif"] {
            let path = PathBuf::from(format!("file.{ext}"));
            assert_eq!(
                ItemType::from_path(&path),
                Some(ItemType::Img),
                "failed for .{ext}"
            )
        }
    }

    #[test]
    fn test_vid_extensions() {
        for ext in ["mp4", "mkv", "mov", "webm", "avi", "m4v"] {
            let path = PathBuf::from(format!("file.{ext}"));
            assert_eq!(
                ItemType::from_path(&path),
                Some(ItemType::Vid),
                "failed for .{ext}"
            )
        }
    }

    #[test]
    fn test_aud_extensions() {
        for ext in ["mp3", "flac", "wav", "ogg", "aac", "m4a", "opus"] {
            let path = PathBuf::from(format!("file.{ext}"));
            assert_eq!(
                ItemType::from_path(&path),
                Some(ItemType::Aud),
                "failed for .{ext}"
            )
        }
    }

    #[test]
    fn test_read_extensions() {
        for ext in ["pdf", "epub"] {
            let path = PathBuf::from(format!("file.{ext}"));
            assert_eq!(
                ItemType::from_path(&path),
                Some(ItemType::Read),
                "failed for .{ext}"
            )
        }
    }

    #[test]
    fn test_reject_ignored_extensions() {
        for ext in ["xzy", "abc", "txt", "docx", "ini", "json"] {
            let path = PathBuf::from(format!("file.{ext}"));
            assert_eq!(ItemType::from_path(&path), None, "failed for .{ext}")
        }
    }
}
