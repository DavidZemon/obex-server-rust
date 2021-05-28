use std::env;
use std::path::PathBuf;

use rocket::http::Status;
use rocket_contrib::json::Json;

use crate::models::{EntryType, TreeEntry};
use crate::response_status::ResponseStatus;
use crate::utils::run;
use std::str::from_utf8;

const DEFAULT_OBEX_ROOT: &str = "/tmp/obex";

pub fn get_tree(root: PathBuf, depth: Option<u32>) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
    let depth = depth.unwrap_or(32 * 1024);

    obex_path()
        .join(root)
        .canonicalize()
        .map_err(|error| ResponseStatus {
            status: Status::BadRequest,
            message: format!("Bad path: {}", error.to_string()),
        })
        .and_then(|computed_root| {
            run(obex_path(), "git", Vec::from(["ls-files"])).map(|output| (computed_root, output))
        })
        .and_then(|(computed_root, output)| {
            if output.status.success() {
                Ok((
                    computed_root,
                    String::from(from_utf8(&output.stdout).unwrap_or("")),
                ))
            } else {
                Err(ResponseStatus {
                    status: Status::InternalServerError,
                    message: format!(
                        "Failed to execute 'git ls-files' due to: {}",
                        from_utf8(&output.stderr)
                            .unwrap_or("Error message not available due to non-UTF8 characters")
                    ),
                })
            }
        })
        .map(|(computed_root, files_output)| {
            let lines: Vec<&str> = files_output.split("\n").collect();
            for line in lines {
                println!("{}", line.trim())
            }

            Json(vec![TreeEntry {
                name: String::from("foo"),
                full_path: String::from(computed_root.to_str().unwrap_or("")),
                entry_type: EntryType::FILE,
                size: Option::None,
                children: Option::None,
                target: Option::None,
            }])
        })
}

fn obex_path() -> PathBuf {
    PathBuf::from(env::var("OBEX_ROOT").unwrap_or(String::from(DEFAULT_OBEX_ROOT)))
}
