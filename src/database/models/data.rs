//! Data models for the core library and entry hierarchy.
//!
//! Contains the primary domain types: [`Library`], [`Entry`], and the type-classifying
//! enums [`LibraryKind`], [`EntryKind`], and [`ItemType`].

use std::path::{Path, PathBuf};
use std::str::FromStr;

use thiserror::Error;

/// Error returned when parsing an unknown [`LibraryKind`] string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("invalid library kind")]
pub struct ParseLibraryKindError;

/// Error returned when parsing an unknown [`EntryKind`] string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("invalid entry kind")]
pub struct ParseEntryKindError;

/// Error returned when parsing an unknown [`ItemType`] string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("invalid item type")]
pub struct ParseItemTypeError;

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
}

impl FromStr for LibraryKind {
    type Err = ParseLibraryKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "movies" => Self::Movies,
            "tv" => Self::Tv,
            "manga" => Self::Manga,
            "books" => Self::Books,
            "audio" => Self::Audio,
            _ => return Err(ParseLibraryKindError),
        })
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
}

impl FromStr for EntryKind {
    type Err = ParseEntryKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "folder" => Self::Folder,
            "file" => Self::File,
            _ => return Err(ParseEntryKindError),
        })
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
        let ext = path.extension()?.to_str()?;

        if ["jpg", "jpeg", "png", "gif", "webp", "avif"]
            .iter()
            .any(|candidate| ext.eq_ignore_ascii_case(candidate))
        {
            return Some(Self::Img);
        }

        if ["mp4", "mkv", "mov", "webm", "avi", "m4v"]
            .iter()
            .any(|candidate| ext.eq_ignore_ascii_case(candidate))
        {
            return Some(Self::Vid);
        }

        if ["mp3", "flac", "wav", "ogg", "aac", "m4a", "opus"]
            .iter()
            .any(|candidate| ext.eq_ignore_ascii_case(candidate))
        {
            return Some(Self::Aud);
        }

        if ["pdf", "epub"]
            .iter()
            .any(|candidate| ext.eq_ignore_ascii_case(candidate))
        {
            return Some(Self::Read);
        }

        None
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
}

impl FromStr for ItemType {
    type Err = ParseItemTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "img" => Self::Img,
            "vid" => Self::Vid,
            "aud" => Self::Aud,
            "read" => Self::Read,
            _ => return Err(ParseItemTypeError),
        })
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

impl Entry {
    /// Returns the file or folder name derived from the entry path.
    pub fn name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
    }

    /// Returns `true` if the entry is eligible for rating.
    ///
    /// Folders are always rateable; image files (e.g. manga pages) are excluded.
    pub fn is_rateable(&self) -> bool {
        match self.kind {
            EntryKind::Folder => true,
            EntryKind::File => !matches!(self.item_type, Some(ItemType::Img)),
        }
    }
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

    #[test]
    fn test_extensions_ignore_case() {
        assert_eq!(
            ItemType::from_path(&PathBuf::from("cover.JPG")),
            Some(ItemType::Img)
        );
        assert_eq!(
            ItemType::from_path(&PathBuf::from("movie.MKV")),
            Some(ItemType::Vid)
        );
        assert_eq!(
            ItemType::from_path(&PathBuf::from("song.FLAC")),
            Some(ItemType::Aud)
        );
        assert_eq!(
            ItemType::from_path(&PathBuf::from("book.PDF")),
            Some(ItemType::Read)
        );
    }

    #[test]
    fn parses_database_strings() {
        assert_eq!(
            "movies".parse::<LibraryKind>().unwrap(),
            LibraryKind::Movies
        );
        assert_eq!("folder".parse::<EntryKind>().unwrap(), EntryKind::Folder);
        assert_eq!("vid".parse::<ItemType>().unwrap(), ItemType::Vid);

        assert!("unknown".parse::<LibraryKind>().is_err());
        assert!("directory".parse::<EntryKind>().is_err());
        assert!("video".parse::<ItemType>().is_err());
    }

    #[test]
    fn entry_name_comes_from_path() {
        let entry = Entry {
            id: 1,
            library_id: 1,
            parent_id: None,
            path: PathBuf::from("Library/Movie/movie.mkv"),
            kind: EntryKind::File,
            item_type: Some(ItemType::Vid),
            size: Some(10),
            mtime: Some(1),
        };

        assert_eq!(entry.name(), "movie.mkv");
    }

    #[test]
    fn entry_rateability_matches_media_type() {
        let folder = Entry {
            id: 1,
            library_id: 1,
            parent_id: None,
            path: PathBuf::from("Library/Movie"),
            kind: EntryKind::Folder,
            item_type: None,
            size: None,
            mtime: Some(1),
        };
        let image = Entry {
            id: 2,
            library_id: 1,
            parent_id: Some(1),
            path: PathBuf::from("Library/Movie/poster.jpg"),
            kind: EntryKind::File,
            item_type: Some(ItemType::Img),
            size: Some(10),
            mtime: Some(1),
        };
        let video = Entry {
            id: 3,
            library_id: 1,
            parent_id: Some(1),
            path: PathBuf::from("Library/Movie/movie.mkv"),
            kind: EntryKind::File,
            item_type: Some(ItemType::Vid),
            size: Some(10),
            mtime: Some(1),
        };

        assert!(folder.is_rateable());
        assert!(!image.is_rateable());
        assert!(video.is_rateable());
    }
}
