#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate cfg_if;

use std::path::PathBuf;

use log::LevelFilter;
use rocket::routes;
use simple_logger::SimpleLogger;
use structopt::StructOpt;

use tree::TreeShaker;

use crate::cmd::Cmd;
use crate::download::Downloader;
use std::time::Duration;

mod cmd;
mod download;
mod models;
mod response_status;
mod routes;
mod tree;

const DEFAULT_OBEX_ROOT: &str = "/tmp/obex";
const DEFAULT_STATIC_CONTENT_ROOT: &str = "./static";

#[derive(StructOpt, Debug)]
#[structopt(name = "obex-server", version = "0.0.1")]
struct CliOpts {
    /// Log level for third-party dependencies
    #[structopt(short, long, default_value = LevelFilter::Info.as_str())]
    level: LevelFilter,

    /// Log level for the obex server
    #[structopt(short, long, default_value = LevelFilter::Debug.as_str())]
    module_level: LevelFilter,

    /// Directory with static content
    #[structopt(short, long, parse(from_os_str), default_value = DEFAULT_STATIC_CONTENT_ROOT)]
    static_content: PathBuf,

    /// Directory of obex contents (if exists, should be root of the git repository)
    #[structopt(short, long, parse(from_os_str), default_value = DEFAULT_OBEX_ROOT)]
    obex_root: PathBuf,

    /// Frequency (in seconds) with which the git repository should be polled for updates
    #[structopt(short, long, default_value = "3000")]
    update: u64,
}

fn main() {
    let args: CliOpts = CliOpts::from_args();

    SimpleLogger::new()
        .with_level(args.level)
        .with_module_level("obex-server-rust", args.module_level)
        .init()
        .unwrap();

    let cmd = Cmd {
        cwd: args.obex_root.clone(),
    };
    init(&args.obex_root, cmd.clone());

    rocket::ignite()
        .manage(TreeShaker {
            obex_path: args.obex_root.clone(),
            cmd,
        })
        .manage(Downloader {
            obex_path: args.obex_root,
        })
        .manage(routes::client::Constants {
            root: args.static_content.clone(),
        })
        .mount(
            "/api/tree",
            routes![routes::tree::get_child_tree, routes::tree::get_root_tree],
        )
        .mount(
            "/api/downloads",
            routes![routes::download::download_root, routes::download::download],
        )
        .mount(
            "/",
            routes::client::get_static_content_routes(args.static_content),
        )
        .launch();
}

fn init(obex_root: &PathBuf, cmd: Cmd) {
    if !obex_root.exists() {
        std::fs::create_dir(obex_root).unwrap();
        let output = cmd
            .run(vec![
                "git",
                "clone",
                "https://github.com/parallaxinc/propeller.git",
                ".",
            ])
            .unwrap();
        let output_text = String::from_utf8(output.stdout).unwrap();
        let error_text = String::from_utf8(output.stderr).unwrap();
        if output.status.success() {
            print!("{}", output_text);
            eprint!("{}", error_text);
        } else {
            panic!("{}", error_text);
        }
    }

    std::thread::spawn(|| {
        let cmd = cmd; // Create a thread-local clone
        #[allow(while_true)]
        while true {
            log::info!("Perform git pull");
            let output = cmd.run(vec!["git", "pull", "--ff-only"]).unwrap();
            let output_text = String::from_utf8(output.stdout).unwrap();
            let error_text = String::from_utf8(output.stderr).unwrap();
            if output.status.success() {
                log::info!("Git pull succeeded (stdout): {}", output_text.trim_end());
                log::info!("Git pull succeeded (stderr): {}", error_text.trim_end());
            } else {
                log::info!("Git failed with: {}", error_text.trim_end());
            }

            std::thread::sleep(Duration::from_secs(60 * 30 /* 30 minutes */));
        }
    });
}
