use std::env;
use std::path::PathBuf;

use rocket::http::Status;
use rocket_contrib::json::Json;

use crate::models::{EntryType, TreeEntry};
use crate::response_status::ResponseStatus;
use crate::utils::run;
use std::fs::{read_dir, read_link};
use std::str::from_utf8;

const DEFAULT_OBEX_ROOT: &str = "/tmp/obex";

pub fn get_tree(root: PathBuf, depth: Option<u32>) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
    let depth = depth.unwrap_or(32 * 1024);

    obex_path()
        .join(root.clone())
        .canonicalize()
        .map_err(|error| ResponseStatus {
            status: Status::BadRequest,
            message: format!("Bad path: {}", error.to_string()),
        })
        .and_then(|absolute_root| {
            run(obex_path(), "git", Vec::from(["ls-files"])).map(|output| (absolute_root, output))
        })
        .and_then(|(absolute_root, output)| {
            if output.status.success() {
                Ok((
                    absolute_root,
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
        .and_then(|(absolute_root, files_in_git)| {
            let lines: Vec<&str> = files_in_git.split("\n").collect();
            match process_tree(absolute_root, root, lines, depth) {
                Ok(trees) => Ok(Json(trees)),
                Err(error) => Err(error),
            }
        })
}

fn obex_path() -> PathBuf {
    PathBuf::from(env::var("OBEX_ROOT").unwrap_or(String::from(DEFAULT_OBEX_ROOT)))
}

fn process_tree(
    absolute_root: PathBuf,
    root: PathBuf,
    include_list: Vec<&str>,
    depth: u32,
) -> Result<Vec<TreeEntry>, ResponseStatus> {
    read_dir(absolute_root)
        .map_err(|error| ResponseStatus {
            status: Status::InternalServerError,
            message: error.to_string(),
        })
        .map(|entries| {
            let mut results: Vec<TreeEntry> = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    let full_path;
                    if root == PathBuf::from(".") {
                        full_path = String::from(entry.file_name().to_str().unwrap_or(""));
                    } else {
                        full_path =
                            String::from(root.join(entry.file_name()).to_str().unwrap_or(""));
                    }

                    if include_list.contains(&full_path.as_str()) {
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_symlink() {
                                results.push(TreeEntry {
                                    name: String::from(entry.file_name().to_str().unwrap()),
                                    full_path,
                                    entry_type: EntryType::SYMLINK,
                                    target: Some(match read_link(entry.path()) {
                                        Ok(target) => String::from(target.to_str().unwrap_or("")),
                                        Err(_) => String::from("INVALID SYMLINK"),
                                    }),
                                    children: None,
                                    size: None,
                                })
                            } else if file_type.is_dir() {
                                // TODO: Implement recursive directory
                            } else if file_type.is_file() {
                                match entry.metadata() {
                                    Ok(metadata) => results.push(TreeEntry {
                                        name: String::from(entry.file_name().to_str().unwrap()),
                                        full_path,
                                        entry_type: EntryType::FILE,
                                        size: Some(metadata.len()),
                                        target: None,
                                        children: None,
                                    }),
                                    Err(error) => {
                                        eprintln!(
                                            "Failed to check size of file {}: {}",
                                            full_path,
                                            error.to_string()
                                        );
                                        results.push(TreeEntry {
                                            name: String::from(entry.file_name().to_str().unwrap()),
                                            full_path,
                                            entry_type: EntryType::FILE,
                                            size: Some(0),
                                            target: None,
                                            children: None,
                                        })
                                    }
                                }
                            }
                        }
                    } else {
                        println!(
                            "Ignoring path because it is not in the whitelist: {}",
                            full_path
                        )
                    }
                }
            }
            results
        })
}
