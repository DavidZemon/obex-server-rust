use std::fs::{read_dir, read_link, FileType};
use std::path::Path;
use std::str::from_utf8;

use rocket::http::Status;
use rocket_contrib::json::Json;

use crate::models::TreeEntry;
use crate::response_status::ResponseStatus;
use crate::utils::run;

pub struct TreeShaker<'a> {
    pub obex_path: &'a Path,
}

impl<'a> TreeShaker<'a> {
    pub fn get_tree(
        &self,
        root: &Path,
        depth: Option<u32>,
    ) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
        let depth = depth.unwrap_or(32 * 1024);

        run(self.obex_path, "git", Vec::from(["ls-files"]))
            .and_then(|output| {
                if output.status.success() {
                    Ok(String::from(from_utf8(&output.stdout).unwrap_or("")))
                } else {
                    Err(ResponseStatus::internal_server_error(format!(
                        "Failed to execute 'git ls-files' due to: {}",
                        from_utf8(&output.stderr)
                            .unwrap_or("Error message not available due to non-UTF8 characters")
                    )))
                }
            })
            .and_then(|files_in_git| {
                let lines: Vec<&str> = files_in_git.split("\n").collect();
                Ok(Json(self.process_tree(&root, &lines, depth)?))
            })
    }

    fn process_tree(
        &self,
        root: &Path,
        include_list: &Vec<&str>,
        depth: u32,
    ) -> Result<Vec<TreeEntry>, ResponseStatus> {
        self.obex_path
            .join(&root)
            .canonicalize()
            .map_err(|error| ResponseStatus {
                status: Status::BadRequest,
                message: format!("Bad path: {}", error.to_string()),
            })
            .and_then(|absolute_root| {
                read_dir(absolute_root).map_err(|error| ResponseStatus::from(error))
            })
            .and_then(|entries| {
                let mut results: Vec<TreeEntry> = Vec::new();
                for dir_entry in entries {
                    let dir_entry = &dir_entry?;
                    let dir_entry_type = dir_entry.file_type()?;
                    let full_path_buf = root.join(dir_entry.file_name());
                    let full_path_str = String::from(full_path_buf.to_str().unwrap_or(""));
                    if TreeShaker::should_generate_tree_entry(
                        dir_entry_type,
                        include_list,
                        full_path_str.as_str(),
                    ) {
                        if dir_entry_type.is_symlink() {
                            results.push(TreeEntry::symlink(
                                String::from(dir_entry.file_name().to_str().unwrap()),
                                full_path_str.clone(),
                                read_link(dir_entry.path())?,
                            ))
                        } else if dir_entry_type.is_dir() {
                            results.push(TreeEntry::folder(
                                String::from(dir_entry.file_name().to_str().unwrap()),
                                full_path_str,
                                if depth == 0 {
                                    None
                                } else {
                                    Some(self.process_tree(
                                        &full_path_buf,
                                        include_list,
                                        depth - 1,
                                    )?)
                                },
                            ))
                        } else if dir_entry_type.is_file() {
                            results.push(TreeEntry::file(
                                String::from(dir_entry.file_name().to_str().unwrap()),
                                full_path_str.clone(),
                                dir_entry.metadata()?.len(),
                            ))
                        }
                    } else {
                        println!("Ignoring directory entry: {}", full_path_str);
                    }
                }
                Ok(results)
            })
    }

    fn should_generate_tree_entry(
        dir_entry_type: FileType,
        include_list: &Vec<&str>,
        full_path: &str,
    ) -> bool {
        full_path != ".git" && (dir_entry_type.is_dir() || include_list.contains(&full_path))
    }
}
