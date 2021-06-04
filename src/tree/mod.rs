use std::fs::{read_dir, read_link};
use std::path::{Path, PathBuf};
use std::str::from_utf8;

use rocket::http::Status;
use rocket_contrib::json::Json;

use crate::models::TreeEntry;
use crate::response_status::ResponseStatus;
use crate::traits::Runner;

pub struct TreeShaker {
    pub obex_path: PathBuf,
    pub runner: Box<dyn Runner>,
}

impl TreeShaker {
    pub fn get_tree(
        &self,
        root: &Path,
        depth: Option<u32>,
    ) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
        let depth = depth.unwrap_or(32 * 1024);

        self.runner
            .run(vec!["git", "ls-files"])
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
                    let entry = &dir_entry?;
                    let entry_name = entry.file_name();
                    let entry_type = entry.file_type()?;
                    let full_path_buf = root.join(&entry_name);
                    let full_path_str = String::from(full_path_buf.to_str().unwrap_or(""));
                    if full_path_str != ".git"
                        && (entry_type.is_dir() || include_list.contains(&full_path_str.as_str()))
                    {
                        if entry_type.is_symlink() {
                            let target = read_link(entry.path())?;
                            results.push(TreeEntry::symlink(entry_name, full_path_str, target))
                        } else if entry_type.is_dir() {
                            results.push(TreeEntry::folder(
                                entry_name,
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
                        } else if entry_type.is_file() {
                            let size = entry.metadata()?.len();
                            results.push(TreeEntry::file(entry_name, full_path_str, size))
                        }
                    } else {
                        log::info!("Ignoring directory entry: {}", full_path_str);
                    }
                }
                Ok(results)
            })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use mockall::predicate;

    use crate::traits::{MockRunner, MockWithStatus, Output};
    use crate::tree::TreeShaker;

    extern crate spectral;

    #[test]
    fn get_tree_failed_git_command() {
        let obex_path = PathBuf::from("/foo/bar");

        let mock_exit_status = MockWithStatus::new();
        mock_exit_status
            .expect_success()
            .times(1)
            .returning(|| true);
        let mock_cmd = MockRunner::new();
        mock_cmd
            .expect_run()
            .with(predicate::eq(vec!["git", "ls-files"]))
            .times(1)
            .returning(|_| {
                Ok(Output {
                    status: mock_exit_status,
                    stdout: vec![],
                    stderr: String::from("Oopsy").into_bytes(),
                })
            });
        let testable = TreeShaker {
            obex_path,
            runner: mock_cmd,
        };
    }
}
