use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize, Clone, Copy, Debug)]
pub enum EntryType {
    FILE,
    SYMLINK,
    FOLDER,
}

#[derive(Serialize, Clone, Debug)]
pub struct TreeEntry {
    pub name: String,
    pub full_path: String,
    pub entry_type: EntryType,
    pub size: Option<u64>,
    pub children: Option<Vec<TreeEntry>>,
    pub target: Option<String>,
}

impl TreeEntry {
    pub fn symlink(name: String, full_path: String, target: PathBuf) -> Self {
        TreeEntry {
            name,
            full_path,
            entry_type: EntryType::SYMLINK,
            size: None,
            children: None,
            target: Some(String::from(target.to_str().unwrap())),
        }
    }

    pub fn folder(name: String, full_path: String, children: Option<Vec<TreeEntry>>) -> Self {
        TreeEntry {
            name,
            full_path,
            entry_type: EntryType::FOLDER,
            size: None,
            children,
            target: None,
        }
    }

    pub fn file(name: String, full_path: String, size: u64) -> Self {
        TreeEntry {
            name,
            full_path,
            entry_type: EntryType::FILE,
            size: Some(size),
            children: None,
            target: None,
        }
    }
}
