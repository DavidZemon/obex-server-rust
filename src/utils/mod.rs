use std::process::{Command, Output};

use rocket::http::Status;

use crate::response_status::ResponseStatus;
use std::path::Path;

pub fn run(cwd: &Path, command: &str, args: Vec<&str>) -> Result<Output, ResponseStatus> {
    Command::new(command)
        .args(&args)
        .current_dir(cwd)
        .output()
        .map_err(|error| ResponseStatus {
            status: Status::InternalServerError,
            message: format!(
                "Failed to execute `'{}' '{}'`: {}",
                command,
                args.join("' '"),
                error.to_string()
            ),
        })
}
