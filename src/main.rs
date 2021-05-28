#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::path::PathBuf;

use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

use tree::get_tree;

use crate::models::TreeEntry;
use crate::response_status::ResponseStatus;

mod models;
mod response_status;
mod tree;
mod utils;

#[get("/tree?<depth>")]
pub fn get_root_tree(depth: Option<u32>) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
    get_tree(PathBuf::from(""), depth)
}

#[get("/tree/<root..>?<depth>")]
pub fn get_child_tree(
    root: PathBuf,
    depth: Option<u32>,
) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
    get_tree(root, depth)
}

fn main() {
    rocket::ignite()
        .mount("/", StaticFiles::from("/static"))
        .mount("/api", routes![get_child_tree, get_root_tree])
        .launch();
}
