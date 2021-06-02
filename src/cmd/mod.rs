use std::path::PathBuf;
use std::process::{Command, Output};

use rocket::http::Status;

use crate::response_status::ResponseStatus;

#[derive(Clone, Debug)]
pub struct Cmd {
    pub cwd: PathBuf,
}

impl Cmd {
    pub fn run(&self, cmd: Vec<&str>) -> Result<Output, ResponseStatus> {
        let (program, args) = cmd
            .split_first()
            .ok_or_else(|| ResponseStatus::internal_server_error(String::from("")))?;
        Command::new(program)
            .args(args)
            .current_dir(self.cwd.as_path())
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
