use serde::Serialize;

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
