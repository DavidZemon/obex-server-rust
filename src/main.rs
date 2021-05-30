#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;

use std::env;
use std::env::VarError;
use std::path::{Path, PathBuf};

use rocket::State;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

use tree::TreeShaker;

use crate::models::TreeEntry;
use crate::response_status::ResponseStatus;
use log::LevelFilter;
use simple_logger::SimpleLogger;

mod models;
mod response_status;
mod tree;
mod utils;

const DEFAULT_OBEX_ROOT: &str = "/tmp/obex";

lazy_static! {
    static ref OBEX_ROOT_OVERRIDE: Result<String, VarError> = env::var("OBEX_ROOT");
}

#[get("/tree?<depth>")]
pub fn get_root_tree(
    depth: Option<u32>,
    tree_shaker: State<TreeShaker>,
) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
    tree_shaker.get_tree(Path::new(""), depth)
}

#[get("/tree/<root..>?<depth>")]
pub fn get_child_tree(
    root: PathBuf,
    depth: Option<u32>,
    tree_shaker: State<TreeShaker>,
) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
    tree_shaker.get_tree(root.as_path(), depth)
}

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level("obex-server-rust", LevelFilter::Debug)
        .init()
        .unwrap();

    let obex_path = Path::new(
        OBEX_ROOT_OVERRIDE
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_else(|_| DEFAULT_OBEX_ROOT),
    );
    rocket::ignite()
        .mount("/", StaticFiles::from("/static"))
        .mount("/api", routes![get_child_tree, get_root_tree])
        .manage(TreeShaker { obex_path })
        .launch();
}
