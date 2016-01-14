use model::WebsiteID;


#[derive(Debug)]
pub enum Event {
    DownloadProcessed {
        wid: WebsiteID,
        resource: bool,
        explored: i32,
    },
    DownloadFailed {
        wid: WebsiteID,
        url: String,
    },
    Terminate,
}
