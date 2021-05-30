#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rocket;

use std::env;
use std::env::VarError;

use log::LevelFilter;
use rocket_contrib::serve::StaticFiles;
use simple_logger::SimpleLogger;

use tree::TreeShaker;

use crate::cmd::Cmd;
use crate::routes::set_api_routes;
use std::path::Path;

mod cmd;
mod models;
mod response_status;
mod routes;
mod tree;

const DEFAULT_OBEX_ROOT: &str = "/tmp/obex";

lazy_static! {
    static ref OBEX_ROOT_OVERRIDE: Result<String, VarError> = env::var("OBEX_ROOT");
}

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level("obex-server-rust", LevelFilter::Debug)
        .init()
        .unwrap();

    let obex_path = Path::new(
        OBEX_ROOT_OVERRIDE
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_else(|_| DEFAULT_OBEX_ROOT),
    );
    set_api_routes(rocket::ignite().mount("/", StaticFiles::from("/static")))
        .manage(TreeShaker {
            obex_path,
            cmd: Cmd { cwd: obex_path },
        })
        .launch();
}
