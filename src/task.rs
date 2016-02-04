use std::fmt;
use std::thread;

use hyper::client::Response;
use mime::Mime;
use time::Timespec;
use url::Url;

use Queues;
use download::task_download;
use model::WebsiteID;
use parser;
use storage;


pub enum Task {
    Download {
        wid: WebsiteID,
        url: String,
        retry: i32,
        resource: bool,
    },
    ParseResource {
        wid: WebsiteID,
        data: Response,
        resource: bool,
    },
    Store {
        wid: WebsiteID,
        url: Url,
        contents: Vec<u8>,
        mime: Option<Mime>,
        timestamp: Timespec,
    },
    Terminate,
}

impl fmt::Debug for Task {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Task::Download { ref wid, ref url, ref retry, ref resource, ..} => {
                fmt.debug_struct("Task::Download")
                   .field("wid", &wid)
                   .field("url", &url)
                   .field("retry", &retry)
                   .field("resource", &resource)
                   .finish()
            }
            Task::ParseResource { ref wid, ref data, ref resource, .. } => {
                fmt.debug_struct("Task::ParseResource")
                   .field("wid", &wid)
                   .field("data",
                          &format!("Response(status={}, url={})", data.status, data.url))
                   .field("resource", &resource)
                   .finish()
            }
            Task::Store { ref wid, ref url, ref contents, ref mime, ref timestamp } => {
                fmt.debug_struct("Task::Store")
                   .field("wid", &wid)
                   .field("url", &format!("{}", url))
                   .field("contents", &format!("Vec<u8>(len={})", contents.len()))
                   .field("mime", &mime)
                   .field("timestamp", &timestamp)
                   .finish()
            }
            Task::Terminate => {
                fmt.debug_struct("Task::Terminate")
                   .finish()
            }
        }
    }
}


pub fn dispatch_task(task: Task, queues: Queues) {
    debug!("Dispatching task in thread {}: {:?}",
           thread::current().name().unwrap_or("<unnamed>".into()),
           task);

    match task {
        Task::Download { wid, url, retry, resource } => {
            task_download(wid, url, retry, resource, queues)
        }
        Task::ParseResource { wid, data, resource } => {
            parser::task_parse_resource(wid, data, resource, queues)
        }
        Task::Store { wid, url, contents, mime, timestamp } => {
            storage::store_resource(wid, url, contents, mime, timestamp, queues)
        }
        Task::Terminate => panic!("Should never be reached"),
    }
}
