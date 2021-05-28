use std::process::{Command, Output};

use rocket::http::Status;

use crate::response_status::ResponseStatus;

pub fn run(command: &str, args: Vec<&str>) -> Result<Output, ResponseStatus> {
    let cloned_args = args.clone();
    Command::new(command)
        .args(args)
        .output()
        .map_err(|error| ResponseStatus {
            status: Status::InternalServerError,
            message: format!(
                "Failed to execute `'{}' '{}'`: {}",
                command,
                cloned_args.join("' '"),
                error.to_string()
            ),
        })
}
