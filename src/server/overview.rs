#![allow(used_underscore_binding)]

use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use router::Router;
use serde_json;


#[derive(Serialize, Deserialize)]
struct OverviewData {
    tags: HashMap<String, TagData>,
    websites: Vec<WebsiteData>,
}


#[derive(Serialize, Deserialize)]
struct WebsiteData {
    title: String,
    url: String,
    tags: Vec<i32>,
}


#[derive(Serialize, Deserialize)]
struct TagData {
    name: String,
    color: Option<String>,
}


pub fn get_router() -> Router {
    let mut router = Router::new();
    router.get("/", handler);

    router
}

fn handler(_: &mut Request) -> IronResult<Response> {
    let data = OverviewData {
        tags: {
            let mut tags: HashMap<String, TagData> = HashMap::new();
            tags.insert("0".into(), TagData { name: "Interessant".into(), color: None });
            tags.insert("1".into(), TagData { name: "Glaube".into(), color: Some("blue".into()) });
            tags
        },
        websites: vec![
            WebsiteData { title: "Doomsday planning for less crazy folk".into(), url: "http://lcamtuf.coredump.cx/prep/".into(), tags: vec![0] },
            WebsiteData { title: "The Long Silence".into(), url: "http://www.ldolphin.org/silence.html".into(), tags: vec![0, 1] },
            WebsiteData { title: "Understanding Depression".into(), url: "http://health.howstuffworks.com/mental-health/depression/facts/understanding-depression-ga.htm".into(), tags: vec![0] },
        ]
    };

    Ok(Response::with((status::Ok, serde_json::to_string(&data).unwrap())))
}
