use std::io::Error as IoError;
#[cfg(not(release))]
use std::path::Path;

use iron::{status, AfterMiddleware};
use iron::modifiers::Header;
use hyper::header::{AccessControlAllowOrigin, ContentType};
use iron::prelude::*;
use logger::Logger;
use logger::format::Format;
use mount::Mount;
use router::NoRoute;
#[cfg(not(release))]
use staticfile::Static;


mod overview;


struct Custom404;

impl AfterMiddleware for Custom404 {
    // For webpack-dev-server which runs on another port
    fn after(&self, _: &mut Request, res: Response) -> IronResult<Response> {
        Ok(res.set((Header(AccessControlAllowOrigin::Any), Header(ContentType::json()))))
    }

    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        if let Some(_) = err.error.downcast::<IoError>() {
            return Ok(Response::with((status::NotFound, "Not Found")));
        }

        if let Some(_) = err.error.downcast::<NoRoute>() {
            return Ok(Response::with((status::NotFound, "Not Found")));
        }

        Err(err)
    }
}


#[cfg(release)]
fn get_assets_router() -> Router {
    include!(concat!(env!("OUT_DIR"), "/server_assets.rs"));
}


#[cfg(not(release))]
fn get_assets_router() -> Static {
    Static::new(Path::new("public/"))
}

pub fn run() {
    let addr = "localhost:3000";

    info!("Starting server at {}", addr);

    // Router
    // let mut router = Router::new();
    // router.get("/", handler);

    // Static files
    let mut mount = Mount::new();
    mount.mount("/api/overview", overview::get_router());
    mount.mount("/", get_assets_router());

    // Logging
    let mut chain = Chain::new(mount);

    let (logger_before, logger_after) = Logger::new(Format::new("{method} {uri} -> {status} \
                                                                 ({response-time})",
                                                                vec![],
                                                                vec![]));
    chain.link_before(logger_before);
    chain.link_after(logger_after);
    chain.link_after(Custom404);

    Iron::new(chain)
        .http(addr)
        .unwrap();
}
