use hyper::client::Response;
use hyper::header::ContentType;
use mime::{self, Mime};
use url::{Url, UrlParser};

use Queues;
use event::Event;
use model::WebsiteID;
use task::Task;


mod css;
mod html;


pub fn task_parse_resource(wid: WebsiteID, mut data: Response, is_resource: bool, queues: Queues) {
    debug!("Parse Resource");

    let content_type: Option<ContentType> = data.headers.get().cloned();

    let explored_resources = match content_type {
        Some(ContentType(Mime(mime::TopLevel::Text, mime::SubLevel::Html, _))) => {
            html::explore_html(&mut data, queues.clone())
        }
        Some(ContentType(Mime(mime::TopLevel::Text, mime::SubLevel::Css, _))) => {
            let base_url = data.url.clone();
            css::explore_css(&mut data, &base_url, queues.clone())
        }
        _ => vec![], // No more exploring, just store in DB
    };

    // Create new download tasks
    for resource in &explored_resources {
        queues.send_task(Task::Download {
            wid: wid,
            url: resource.serialize(),
            retry: 0,
            resource: true,
        })
    }

    // Tell the main event handler we're done with processing
    queues.send_event(Event::DownloadProcessed {
        wid: wid,
        resource: is_resource,
        explored: explored_resources.len() as i32,
    });
}


fn resolve_rel_url(base: &Url, url: &str) -> Option<Url> {
    UrlParser::new().base_url(base).parse(url).ok()
}
