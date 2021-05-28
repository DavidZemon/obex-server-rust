use std::io::{Cursor, Error};

use chrono::{DateTime, Utc};
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::{response, Request, Response};
use serde_json::json;
use std::fmt::Display;
use std::time::SystemTime;

#[derive(Debug)]
pub struct ResponseStatus {
    pub status: Status,
    pub message: String,
}

impl ResponseStatus {
    pub fn internal_server_error(message: String) -> Self {
        ResponseStatus {
            status: Status::InternalServerError,
            message,
        }
    }

    pub fn from<T: Display>(error: T) -> Self {
        ResponseStatus::internal_server_error(error.to_string())
    }
}

impl std::convert::From<std::io::Error> for ResponseStatus {
    fn from(error: Error) -> Self {
        ResponseStatus::internal_server_error(error.to_string())
    }
}

impl<'r> Responder<'r> for ResponseStatus {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        let now: DateTime<Utc> = SystemTime::now().into();
        if self.status == Status::InternalServerError {
            eprintln!("An unexpected error occurred: {}", self.message)
        }

        Response::build()
            .status(self.status)
            .header(ContentType::JSON)
            .sized_body(Cursor::new(
                json!({
                    "code": self.status.code,
                    "reason": self.status.reason,
                    "message": self.message,
                    "timestamp": now.to_rfc3339(),
                })
                .to_string(),
            ))
            .ok()
    }
}
