use std::env;
use std::fs::{read_dir, read_link, DirEntry, FileType, Metadata};
use std::path::PathBuf;
use std::str::from_utf8;

use rocket::http::Status;
use rocket_contrib::json::Json;

use crate::models::{EntryType, TreeEntry};
use crate::response_status::ResponseStatus;
use crate::utils::run;

const DEFAULT_OBEX_ROOT: &str = "/tmp/obex";

pub fn get_tree(root: PathBuf, depth: Option<u32>) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
    let depth = depth.unwrap_or(32 * 1024);

    obex_path()
        .join(&root)
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
            Ok(Json(process_tree(&absolute_root, &root, &lines, depth)?))
        })
}

fn obex_path() -> PathBuf {
    PathBuf::from(env::var("OBEX_ROOT").unwrap_or(String::from(DEFAULT_OBEX_ROOT)))
}

fn process_tree(
    absolute_root: &PathBuf,
    root: &PathBuf,
    include_list: &Vec<&str>,
    depth: u32,
) -> Result<Vec<TreeEntry>, ResponseStatus> {
    read_dir(absolute_root)
        .map_err(|error| ResponseStatus::from(error))
        .and_then(|entries| {
            let mut results: Vec<TreeEntry> = Vec::new();
            for dir_entry in entries {
                process_dir_entry(&dir_entry?, absolute_root, root, include_list, depth)?
                    .map(|tree_entry| results.push(tree_entry));
            }
            Ok(results)
        })
}

fn process_dir_entry(
    dir_entry: &DirEntry,
    absolute_root: &PathBuf,
    root: &PathBuf,
    include_list: &Vec<&str>,
    depth: u32,
) -> Result<Option<TreeEntry>, ResponseStatus> {
    let dir_entry_type = dir_entry.file_type()?;
    let full_path = root.join(dir_entry.file_name());
    let full_path_str = String::from(full_path.to_str().unwrap_or(""));

    return if should_generate_tree_entry(dir_entry_type, include_list, full_path_str.clone()) {
        if dir_entry_type.is_symlink() {
            Ok(Some(build_symlink_entry(&dir_entry, full_path_str)))
        } else if dir_entry_type.is_dir() {
            Ok(Some(build_dir_tree_entry(
                absolute_root,
                include_list,
                depth,
                &dir_entry,
                &full_path,
                full_path_str,
            )?))
        } else if dir_entry_type.is_file() {
            Ok(Some(build_file_entry(
                &dir_entry,
                full_path_str,
                dir_entry.metadata()?,
            )))
        } else {
            Ok(None)
        }
    } else {
        println!("Ignoring directory entry: {}", full_path_str);
        Ok(None)
    };
}

fn should_generate_tree_entry(
    dir_entry_type: FileType,
    include_list: &Vec<&str>,
    full_path_str: String,
) -> bool {
    full_path_str != ".git"
        && (dir_entry_type.is_dir() || include_list.contains(&full_path_str.as_str()))
}

fn build_symlink_entry(entry: &DirEntry, full_path: String) -> TreeEntry {
    TreeEntry {
        name: String::from(entry.file_name().to_str().unwrap()),
        full_path,
        entry_type: EntryType::SYMLINK,
        target: Some(match read_link(entry.path()) {
            Ok(target) => String::from(target.to_str().unwrap_or("")),
            Err(_) => String::from("INVALID SYMLINK"),
        }),
        children: None,
        size: None,
    }
}

fn build_dir_tree_entry(
    absolute_root: &PathBuf,
    include_list: &Vec<&str>,
    depth: u32,
    entry: &DirEntry,
    full_path: &PathBuf,
    full_path_str: String,
) -> Result<TreeEntry, ResponseStatus> {
    Ok(TreeEntry {
        name: String::from(entry.file_name().to_str().unwrap()),
        full_path: full_path_str,
        entry_type: EntryType::FOLDER,
        children: if depth == 0 {
            None
        } else {
            Some(process_tree(
                &absolute_root.join(entry.path()),
                &full_path,
                include_list,
                depth - 1,
            )?)
        },
        target: None,
        size: None,
    })
}

fn build_file_entry(entry: &DirEntry, full_path: String, metadata: Metadata) -> TreeEntry {
    TreeEntry {
        name: String::from(entry.file_name().to_str().unwrap()),
        full_path,
        entry_type: EntryType::FILE,
        size: Some(metadata.len()),
        target: None,
        children: None,
    }
}
