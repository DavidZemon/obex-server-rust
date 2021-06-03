use std::path::{Path, PathBuf};

use rocket::http::uri::{SegmentError, Segments};
use rocket::http::{ContentType, Status};
use rocket::request::FromSegments;
use rocket::response::{NamedFile, Responder};
use rocket::{response, Request};

use crate::response_status::ResponseStatus;

pub struct Downloader {
    pub obex_path: PathBuf,
}

pub struct UnsafePathBuf(pub PathBuf);

impl<'a> FromSegments<'a> for UnsafePathBuf {
    type Error = SegmentError;

    fn from_segments(segments: Segments<'a>) -> Result<UnsafePathBuf, SegmentError> {
        Ok(UnsafePathBuf {
            0: segments.into_path_buf(true)?,
        })
    }
}

pub struct BetterNamedFile(pub NamedFile);

impl<'r> Responder<'r> for BetterNamedFile {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        let mut response = self.0.respond_to(req)?;
        if !response.headers().contains("Content-Type") {
            response.set_header(ContentType::new("text", "plain"));
        }

        Ok(response)
    }
}

impl Downloader {
    pub fn download(&self, path: PathBuf) -> Result<Option<BetterNamedFile>, ResponseStatus> {
        let relative_path_str =
            urlencoding::decode(path.to_str().unwrap_or("/")).map_err(ResponseStatus::from)?;
        let absolute_path = self.obex_path.join(Path::new(relative_path_str.as_str()));

        if relative_path_str.ends_with(".zip") && absolute_path.is_dir() {
            self.get_dir(absolute_path)
        } else if absolute_path.exists() {
            Ok(match NamedFile::open(absolute_path).ok() {
                Some(f) => Some(BetterNamedFile { 0: f }),
                None => None,
            })
        } else if relative_path_str.ends_with(".zip") {
            Err(ResponseStatus {
                status: Status::BadRequest,
                message: format!(
                    "No such file ({}) or direcotry ({}",
                    relative_path_str,
                    relative_path_str.split_at(relative_path_str.len() - 4).0
                ),
            })
        } else {
            Err(ResponseStatus {
                status: Status::BadRequest,
                message: format!("No such file {}", relative_path_str),
            })
        }
    }

    fn get_dir(&self, absolute_path: PathBuf) -> Result<Option<BetterNamedFile>, ResponseStatus> {
        Ok(None)
    }
}
