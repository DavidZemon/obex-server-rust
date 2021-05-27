#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use crate::EntryType::FILE;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
enum EntryType {
    FILE,
    SYMLINK,
    FOLDER,
}

#[derive(Serialize)]
struct TreeEntry {
    name: String,
    full_path: String,
    entry_type: EntryType,
    size: Option<u64>,
    children: Option<Vec<TreeEntry>>,
    target: Option<String>,
}

#[get("/api/tree/<root..>")]
fn tree(root: PathBuf) -> rocket_contrib::json::Json<Vec<TreeEntry>> {
    rocket_contrib::json::Json(vec![TreeEntry {
        name: String::from("foo"),
        full_path: String::from(root.to_str().unwrap_or("")),
        entry_type: FILE,
        size: Option::None,
        children: Option::None,
        target: Option::None,
    }])
}

fn main() {
    rocket::ignite().mount("/", routes![tree]).launch();
}
