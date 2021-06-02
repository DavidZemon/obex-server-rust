pub mod client {
    use std::path::PathBuf;

    use rocket::response::NamedFile;
    use rocket::State;
    use rocket::{get, routes, Route};

    pub struct Constants {
        pub root: PathBuf,
    }

    pub fn get_static_content_routes(root: PathBuf) -> Vec<Route> {
        let root_str = root.to_str().unwrap();
        if root.exists() {
            log::info!("Serving static content from {}", root_str);
            routes![root, default]
        } else {
            log::info!(
                "Static content directory not found ({}). Serving API only.",
                root_str
            );
            routes![]
        }
    }

    #[get("/")]
    fn root(state: State<Constants>) -> Option<NamedFile> {
        serve_file(state.root.join(PathBuf::from("index.html")))
    }

    #[get("/<requested_path..>")]
    fn default(requested_path: PathBuf, state: State<Constants>) -> Option<NamedFile> {
        let resolved_path = state.root.join(requested_path);
        if resolved_path.exists() {
            serve_file(resolved_path)
        } else {
            root(state)
        }
    }

    fn serve_file(path: PathBuf) -> Option<NamedFile> {
        NamedFile::open(path).ok()
    }
}

pub mod tree {
    use std::path::PathBuf;

    use rocket::get;
    use rocket::State;
    use rocket_contrib::json::Json;

    use crate::models::TreeEntry;
    use crate::response_status::ResponseStatus;
    use crate::tree::TreeShaker;

    #[get("/?<depth>")]
    pub fn get_root_tree(
        depth: Option<u32>,
        tree_shaker: State<TreeShaker>,
    ) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
        tree_shaker.get_tree(&PathBuf::from(""), depth)
    }

    #[get("/<root..>?<depth>")]
    pub fn get_child_tree(
        root: PathBuf,
        depth: Option<u32>,
        tree_shaker: State<TreeShaker>,
    ) -> Result<Json<Vec<TreeEntry>>, ResponseStatus> {
        tree_shaker.get_tree(&root, depth)
    }
}
