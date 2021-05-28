#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::env;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use rocket::response::status::BadRequest;
use rocket_contrib::json::Json;

use models::{EntryType, TreeEntry};

mod models;
mod responder;

const DEFAULT_OBEX_ROOT: &str = "/tmp/obex";

fn from_root(path: PathBuf) -> io::Result<PathBuf> {
    let obex_root = PathBuf::from(env::var("OBEX_ROOT").unwrap_or(String::from("/tmp/obex")));
    obex_root.join(path).canonicalize()
}

#[get("/api/tree/<root..>?<depth>")]
fn tree(root: PathBuf, depth: Option<u32>) -> Result<Json<Vec<TreeEntry>>, BadRequest<String>> {
    let depth = depth.unwrap_or(32 * 1024);

    let computed_root_result = from_root(root)
        .map_err(|error| BadRequest(Some(format!("Bad path: {}", error.to_string()))));

    return if computed_root_result.is_err() {
        Err(computed_root_result.err().unwrap())
    } else {
        Command::new("git")
            .arg("ls-files")
            .output()
            .map(|_| {
                Json(vec![TreeEntry {
                    name: String::from("foo"),
                    full_path: String::from(computed_root_result.unwrap().to_str().unwrap_or("")),
                    entry_type: EntryType::FILE,
                    size: Option::None,
                    children: Option::None,
                    target: Option::None,
                }])
            })
            // FIXME: BadRequest is the wrong type - it should be 500
            .map_err(|error| BadRequest(Some(format!("Dumb failure: {}", error.to_string()))))
    };
}

#[get("/api/tree?<depth>")]
fn tree_root(depth: Option<u32>) -> Result<Json<Vec<TreeEntry>>, BadRequest<String>> {
    tree(PathBuf::from("/"), depth)
}

fn main() {
    rocket::ignite()
        .mount("/", routes![tree, tree_root])
        .launch();
}
