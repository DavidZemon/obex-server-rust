use std::path::Path;
use std::process::{Command, Output};

use rocket::http::Status;

use crate::response_status::ResponseStatus;

pub struct Cmd<'a> {
    pub cwd: &'a Path,
}

impl<'a> Cmd<'a> {
    pub fn run(&self, cmd: Vec<&str>) -> Result<Output, ResponseStatus> {
        let (program, args) = cmd
            .split_first()
            .ok_or_else(|| ResponseStatus::internal_server_error(String::from("")))?;
        Command::new(program)
            .args(args)
            .current_dir(self.cwd)
            .output()
            .map_err(|error| ResponseStatus {
                status: Status::InternalServerError,
                message: format!(
                    "Failed to execute `'{}' '{}'`: {}",
                    format!("'{}'", cmd.join("' '")),
                    args.join("' '"),
                    error.to_string()
                ),
            })
    }
}
