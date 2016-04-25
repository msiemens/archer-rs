#![allow(used_underscore_binding)]

use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use router::Router;
use serde_json;

use storage;


#[derive(Serialize, Deserialize)]
pub struct OverviewData {
    pub tags: HashMap<String, TagData>,
    pub websites: Vec<WebsiteData>,
}


#[derive(Serialize, Deserialize)]
pub struct WebsiteData {
    pub title: String,
    pub url: String,
    pub tags: Vec<i32>,
}


#[derive(Serialize, Deserialize)]
pub struct TagData {
    pub name: String,
    pub color: Option<String>,
}


pub fn get_router() -> Router {
    let mut router = Router::new();
    router.get("/", handler);

    router
}

fn handler(_: &mut Request) -> IronResult<Response> {
    let data = storage::get_overview();

    Ok(Response::with((status::Ok, serde_json::to_string(&data).unwrap())))
}
