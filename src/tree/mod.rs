use std::env;
use std::path::PathBuf;

use rocket::http::Status;
use rocket_contrib::json::Json;

use crate::models::{EntryType, TreeEntry};
use crate::response_status::ResponseStatus;
use crate::utils::run;

const DEFAULT_OBEX_ROOT: &str = "/tmp/obex";

pub fn get_tree(root: PathBuf, depth: Option<u32>) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
    let depth = depth.unwrap_or(32 * 1024);

    PathBuf::from(env::var("OBEX_ROOT").unwrap_or(String::from(DEFAULT_OBEX_ROOT)))
        .join(root)
        .canonicalize()
        .map_err(|error| ResponseStatus {
            status: Status::BadRequest,
            message: format!("Bad path: {}", error.to_string()),
        })
        .and_then(|computed_root| run("git", Vec::from(["ls-files"])).map(|_| computed_root))
        .map(|computed_root_result| {
            Json(vec![TreeEntry {
                name: String::from("foo"),
                full_path: String::from(computed_root_result.to_str().unwrap_or("")),
                entry_type: EntryType::FILE,
                size: Option::None,
                children: Option::None,
                target: Option::None,
            }])
        })
}
