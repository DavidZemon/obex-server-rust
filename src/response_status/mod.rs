use std::error::Error;
use std::io::Cursor;

use chrono::{DateTime, Utc};
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::{response, Request, Response};
use serde_json::json;
use std::time::SystemTime;

#[derive(Debug)]
pub struct ResponseStatus {
    pub status: Status,
    pub message: String,
}

impl ResponseStatus {
    pub fn from(error: Box<dyn Error>) -> Self {
        ResponseStatus {
            status: Status::InternalServerError,
            message: error.to_string(),
        }
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
