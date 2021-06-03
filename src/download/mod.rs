use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use rocket::http::hyper::header::{Charset, ContentDisposition, DispositionParam, DispositionType};
use rocket::http::uri::{SegmentError, Segments};
use rocket::http::{ContentType, Status};
use rocket::request::FromSegments;
use rocket::response::{NamedFile, Responder};
use rocket::{response, Request, Response};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

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

pub struct DownloadResponse {
    pub path: Option<PathBuf>,
    pub data: Option<Vec<u8>>,
}

impl DownloadResponse {
    pub fn from(path: PathBuf) -> Self {
        DownloadResponse {
            path: Some(path),
            data: None,
        }
    }

    pub fn from_zip(path: PathBuf, data: Vec<u8>) -> Self {
        DownloadResponse {
            path: Some(path),
            data: Some(data),
        }
    }
}

impl<'r> Responder<'r> for DownloadResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        if self.data.is_some() {
            Response::build()
                .header(ContentType::ZIP)
                .header(ContentDisposition {
                    disposition: DispositionType::Attachment,
                    parameters: vec![DispositionParam::Filename(
                        Charset::Us_Ascii,
                        None,
                        Vec::from(self.path.unwrap().file_name().unwrap().to_str().unwrap()),
                    )],
                })
                .sized_body(Cursor::new(self.data.unwrap()))
                .ok()
        } else if self.path.is_some() {
            let mut response = NamedFile::open(self.path.unwrap())
                .ok()
                .unwrap()
                .respond_to(req)?;
            if !response.headers().contains("Content-Type") {
                response.set_header(ContentType::new("text", "plain"));
            }
            Ok(response)
        } else {
            Err(Status::InternalServerError)
        }
    }
}

impl Downloader {
    pub fn download(&self, path: PathBuf) -> Result<DownloadResponse, ResponseStatus> {
        let relative_path_str =
            urlencoding::decode(path.to_str().unwrap_or("/")).map_err(ResponseStatus::from)?;
        let absolute_path = self.obex_path.join(Path::new(relative_path_str.as_str()));

        let abs_path_without_zip =
            PathBuf::from(absolute_path.to_str().unwrap().trim_end_matches(".zip"));

        if relative_path_str.ends_with(".zip") && abs_path_without_zip.is_dir() {
            self.get_dir(abs_path_without_zip, absolute_path)
        } else if absolute_path.exists() {
            Ok(DownloadResponse::from(absolute_path))
        } else if relative_path_str.ends_with(".zip") {
            Err(ResponseStatus {
                status: Status::BadRequest,
                message: format!(
                    "No such file ({}) or directory ({}",
                    relative_path_str,
                    abs_path_without_zip.to_str().unwrap()
                ),
            })
        } else {
            Err(ResponseStatus {
                status: Status::BadRequest,
                message: format!("No such file {}", relative_path_str),
            })
        }
    }

    fn get_dir(
        &self,
        directory: PathBuf,
        requested_path: PathBuf,
    ) -> Result<DownloadResponse, ResponseStatus> {
        let mut data: Vec<u8> = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut data));
            for entry in WalkDir::new(&directory) {
                let entry = entry.map_err(ResponseStatus::from)?;
                if !entry.path().is_dir() {
                    let mut next_file = File::open(entry.path())?;

                    zip.start_file(
                        entry
                            .path()
                            .strip_prefix(directory.as_path())
                            .unwrap()
                            .to_str()
                            .unwrap(),
                        FileOptions::default(),
                    )
                    .map_err(ResponseStatus::from)?;

                    let mut entry_data: Vec<u8> = Vec::new();
                    next_file.read_to_end(&mut entry_data)?;
                    zip.write(entry_data.as_slice())?;
                }
            }
            zip.finish().map_err(ResponseStatus::from)?;
        }
        Ok(DownloadResponse::from_zip(requested_path, data))
    }
}
