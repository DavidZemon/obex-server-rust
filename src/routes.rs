use std::path::{Path, PathBuf};

use rocket::{Rocket, State};
use rocket_contrib::json::Json;

use crate::models::TreeEntry;
use crate::response_status::ResponseStatus;
use crate::tree::TreeShaker;

pub fn set_api_routes(instance: Rocket) -> Rocket {
    instance.mount("/api", routes![get_child_tree, get_root_tree])
}

#[get("/tree?<depth>")]
fn get_root_tree(
    depth: Option<u32>,
    tree_shaker: State<TreeShaker>,
) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
    tree_shaker.get_tree(Path::new(""), depth)
}

#[get("/tree/<root..>?<depth>")]
fn get_child_tree(
    root: PathBuf,
    depth: Option<u32>,
    tree_shaker: State<TreeShaker>,
) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
    tree_shaker.get_tree(root.as_path(), depth)
}
