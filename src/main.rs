#![feature(proc_macro_hygiene, decl_macro)]

use std::path::PathBuf;

use log::LevelFilter;
use rocket::routes;
use simple_logger::SimpleLogger;
use structopt::StructOpt;

use tree::TreeShaker;

use crate::cmd::Cmd;

mod cmd;
mod models;
mod response_status;
mod routes;
mod tree;

const DEFAULT_OBEX_ROOT: &str = "/tmp/obex";
const DEFAULT_STATIC_CONTENT_ROOT: &str = "./static";

#[derive(StructOpt, Debug)]
#[structopt(name = "obex-server")]
struct CliOpts {
    #[structopt(short, long, default_value = LevelFilter::Info.as_str())]
    level: LevelFilter,

    #[structopt(short, long, default_value = LevelFilter::Debug.as_str())]
    module_level: LevelFilter,

    #[structopt(short, long, parse(from_os_str), default_value = DEFAULT_STATIC_CONTENT_ROOT)]
    static_content: PathBuf,

    #[structopt(short, long, parse(from_os_str), default_value = DEFAULT_OBEX_ROOT)]
    obex_root: PathBuf,
}

fn main() {
    let args: CliOpts = CliOpts::from_args();

    SimpleLogger::new()
        .with_level(args.level)
        .with_module_level("obex-server-rust", args.module_level)
        .init()
        .unwrap();

    rocket::ignite()
        .manage(TreeShaker {
            obex_path: args.obex_root.clone(),
            cmd: Cmd {
                cwd: args.obex_root,
            },
        })
        .manage(routes::client::Constants {
            root: args.static_content.clone(),
        })
        .mount(
            "/api/tree",
            routes![routes::tree::get_child_tree, routes::tree::get_root_tree],
        )
        .mount(
            "/",
            routes::client::get_static_content_routes(args.static_content),
        )
        .launch();
}
