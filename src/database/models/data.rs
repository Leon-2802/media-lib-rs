use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryKind {
    #[default]
    Movies,
    Tv,
    Manga,
    Books,
    Music,
    Audiobooks,
}

impl LibraryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Movies => "movies",
            Self::Tv => "tv",
            Self::Manga => "manga",
            Self::Books => "books",
            Self::Music => "music",
            Self::Audiobooks => "audiobooks",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "movies" => Some(Self::Movies),
            "tv" => Some(Self::Tv),
            "manga" => Some(Self::Manga),
            "books" => Some(Self::Books),
            "music" => Some(Self::Music),
            "audiobooks" => Some(Self::Audiobooks),
            _ => None,
        }
    }

    pub fn from(s: String) -> Option<Self> {
        match s.as_str() {
            "movies" => Some(Self::Movies),
            "tv" => Some(Self::Tv),
            "manga" => Some(Self::Manga),
            "books" => Some(Self::Books),
            "music" => Some(Self::Music),
            "audiobooks" => Some(Self::Audiobooks),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntryKind {
    #[default]
    Folder,
    File,
}

impl EntryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Folder => "folder",
            Self::File => "file",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "folder" => Some(Self::Folder),
            "file" => Some(Self::File),
            _ => None,
        }
    }

    pub fn from(s: String) -> Option<Self> {
        match s.as_str() {
            "folder" => Some(Self::Folder),
            "file" => Some(Self::File),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItemType {
    #[default]
    Img,
    Vid,
    Aud,
    Read,
}

impl ItemType {
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

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Img => "img",
            Self::Vid => "vid",
            Self::Aud => "aud",
            Self::Read => "read",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "img" => Some(Self::Img),
            "vid" => Some(Self::Vid),
            "aud" => Some(Self::Aud),
            "read" => Some(Self::Read),
            _ => None,
        }
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    pub id: i64,
    pub name: String,
    pub path: PathBuf,
    pub kind: LibraryKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: i64,
    pub library_id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub path: PathBuf,
    pub kind: EntryKind,
    pub item_type: Option<ItemType>,
    pub size: Option<i64>,
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
