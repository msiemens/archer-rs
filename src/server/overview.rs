#![allow(used_underscore_binding)]

use std::collections::HashMap;

use bodyparser;
use iron::prelude::*;
use iron::status;
use router::Router;
use serde_json;

use storage;


#[derive(Debug, Serialize, Deserialize)]
pub struct OverviewData {
    pub tags: HashMap<String, TagData>,
    pub websites: Vec<WebsiteData>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct WebsiteData {
    pub title: String,
    pub url: String,
    pub tags: Vec<i32>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct TagData {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnqueueWebsite {
    pub title: String,
    pub url: String,
    pub tags: String
}


pub fn get_router() -> Router {
    let mut router = Router::new();
    router.get("/", handler_index);
    router.post("/", handler_enqueue);

    router
}


fn handler_index(_: &mut Request) -> IronResult<Response> {
    let data = storage::get_overview();

    Ok(Response::with((status::Ok, serde_json::to_string(&data).unwrap())))
}


fn handler_enqueue(req: &mut Request) -> IronResult<Response> {
    if let Ok(Some(website)) = req.get::<bodyparser::Struct<EnqueueWebsite>>() {
        storage::enqueue_website(website.title, website.url, website.tags);
        Ok(Response::with((status::Ok, "{\"success\": true}")))
    } else {
        Ok(Response::with(status::BadRequest))
    }
}
