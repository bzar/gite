extern crate structopt;
use structopt::StructOpt;

#[macro_use]
extern crate log;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use git2::{Blob, ObjectType, Repository};
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};

const INDEX_FILE: &str = "index.html";

#[derive(Debug, StructOpt)]
#[structopt(name = "gite", about = "A Git-backed HTTP server.")]
struct Opt {
    #[structopt(short = "d", long = "debug")]
    debug: bool,
    #[structopt(short = "r", long = "revision", default_value = "master")]
    revision: String,
    #[structopt(parse(from_os_str), default_value=".")]
    path: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let level = if opt.debug { log::Level::Trace } else { log::Level::Info };
    simple_logger::init_with_level(level).expect("Error initializing logger");

    let repo = Arc::new(Mutex::new(Repository::open(opt.path).expect("Repository not found")));
    let revision = Arc::new(opt.revision);

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr)
        .serve(move || {
            let repo = Arc::clone(&repo);
            let revision = Arc::clone(&revision);
            service_fn_ok(move |req| {
                serve(req, &repo.clone().lock().unwrap(), &revision)
            })
        })
        .map_err(|e| error!("Server error: {}", e));
    hyper::rt::run(server);
}

fn serve(req: Request<Body>, repo: &Repository, revision: &str) -> Response<Body> {
    let path = {
        let path = Path::new(req.uri().path());
        if path == Path::new("/") {
            Path::new(INDEX_FILE)
        } else {
            path.strip_prefix("/").unwrap()
        }
    };
    let blob = retrieve_blob(&repo, revision, path);
    match blob {
        Ok(blob) => {
            info!("200 {} {}", req.method().as_str(), req.uri());
            Response::new(Body::from(blob.content().to_owned()))
        },
        Err(e) => {
            info!("404 {} {}", req.method().as_str(), req.uri());
            warn!("{}", e.message());
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap()
        }
    }
}

fn retrieve_blob<'a>(repo: &'a Repository, revision: &str, path: &Path) -> Result<Blob<'a>, git2::Error> {
    let obj = repo.revparse_single(revision)?
        .peel_to_tree()?
        .get_path(path)?
        .to_object(&repo)?;

    match obj.kind() {
        Some(ObjectType::Blob) => obj.peel_to_blob(),
        Some(ObjectType::Tree) => {
            obj.as_tree().unwrap()
                .get_path(Path::new(INDEX_FILE))?
                .to_object(&repo)?
                .peel_to_blob()
        }
        _ => Err(git2::Error::from_str("Path does not point to a blob nor a tree"))
    }
}
