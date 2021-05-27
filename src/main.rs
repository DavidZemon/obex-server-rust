#![feature(proc_macro_hygiene, decl_macro)]

mod models;

use models::{EntryType, TreeEntry};
use std::path::PathBuf;

#[macro_use]
extern crate rocket;

#[get("/api/tree/<root..>")]
fn tree(root: PathBuf) -> rocket_contrib::json::Json<Vec<TreeEntry>> {
    rocket_contrib::json::Json(vec![TreeEntry {
        name: String::from("foo"),
        full_path: String::from(root.to_str().unwrap_or("")),
        entry_type: EntryType::FILE,
        size: Option::None,
        children: Option::None,
        target: Option::None,
    }])
}

fn main() {
    rocket::ignite().mount("/", routes![tree]).launch();
}
