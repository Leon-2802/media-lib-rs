use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryKind {
    Movies,
    Tv,
    Manga,
    Books,
    Music,
    Audiobooks,
}

impl LibraryKind {
    pub fn as_str(&self) -> &'static str {
        todo!()
    }

    pub fn from_str(s: &str) -> Option<Self> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Folder,
    File,
}

impl EntryKind {
    pub fn as_str(&self) -> &'static str {
        todo!()
    }

    pub fn from_str(s: &str) -> Option<Self> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
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
        todo!()
    }

    pub fn from_str(s: &str) -> Option<Self> {
        todo!()
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
