use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    pub id: i32,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Series {
    pub id: i32,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub id: i32,
    pub series_id: i32,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemType {
    IMG,
    VID,
    AUD,
    READ,
}

impl ItemType {
    pub fn from_path(path: &PathBuf) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "avif" => Some(Self::IMG),
            "mp4" | "mkv" | "mov" | "webm" | "avi" | "m4v" => Some(Self::VID),
            "mp3" | "flac" | "wav" | "ogg" | "aac" | "m4a" | "opus" => Some(Self::AUD),
            "pdf" | "epub" => Some(Self::READ),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: i32,
    pub part_id: i32,
    pub name: String,
    pub path: PathBuf,
    pub item_type: ItemType,
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
                Some(ItemType::IMG),
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
                Some(ItemType::VID),
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
                Some(ItemType::AUD),
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
                Some(ItemType::READ),
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
