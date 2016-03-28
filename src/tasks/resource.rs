use mime::Mime;
use time::Timespec;
use url::Url;
use hyper::header::ContentType;


use download::Response;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stage {
    Waiting,
    Download,
    Parse,
    Store,
    Done,
}


#[derive(Debug)]
pub struct DetailsStorage<'a> {
    pub contents: &'a [u8],
    pub mime: &'a Option<Mime>,
    pub timestamp: &'a Timespec,
}


#[derive(Debug)]
pub struct Resource {
    stage: Stage,
    failed: bool,

    url: Url,
    retries: i32,
    response: Option<Response>,
    timestamp: Option<Timespec>,

    contents: Vec<u8>,
    mime: Option<Mime>,
}

impl Resource {
    pub fn new(url: Url) -> Resource {
        Resource {
            stage: Stage::Waiting,
            failed: false,
            url: url,
            retries: 0,
            response: None,
            timestamp: None,
            contents: Vec::new(),
            mime: None,
        }
    }

    pub fn get_stage(&self) -> Stage {
        self.stage
    }

    pub fn is_failed(&self) -> bool {
        self.failed
    }

    // Lifecycle methods

    pub fn failed(&mut self) {
        self.failed = true;
    }

    pub fn skip(&mut self) {
        assert!(!self.failed);

        self.stage = Stage::Done;
    }

    pub fn start_download(&mut self) {
        assert!(!self.failed);
        assert_eq!(self.stage, Stage::Waiting);

        self.stage = Stage::Download;
    }

    pub fn retry_download(&mut self) {
        assert!(!self.failed);
        assert_eq!(self.stage, Stage::Download);

        self.retries += 1;
    }

    pub fn downloaded(&mut self, response: Response, timestamp: Timespec) {
        assert!(!self.failed);
        assert_eq!(self.stage, Stage::Download);

        self.response = Some(response);
        self.timestamp = Some(timestamp);

        self.stage = Stage::Parse;
    }

    pub fn parsed(&mut self, contents: Vec<u8>, mime: Option<Mime>) {
        assert!(!self.failed);
        assert_eq!(self.stage, Stage::Parse);

        self.contents = contents;
        self.mime = mime;

        self.stage = Stage::Store;
    }

    pub fn stored(&mut self) {
        assert!(!self.failed);
        assert_eq!(self.stage, Stage::Store);

        self.stage = Stage::Done;
    }

    // Details accessor methods

    pub fn get_url(&self) -> &Url {
        &self.url
    }

    pub fn get_retries(&self) -> i32 {
        assert!(!self.failed);
        assert_eq!(self.stage, Stage::Download);

        self.retries
    }

    pub fn get_storage_details(&self) -> DetailsStorage {
        assert!(!self.failed);
        assert_eq!(self.stage, Stage::Store);

        DetailsStorage {
            contents: &self.contents,
            mime: &self.mime,
            timestamp: self.timestamp.as_ref().unwrap(),
        }
    }

    // Resource accessor methods

    pub fn get_content_type(&self) -> Option<ContentType> {
        assert!(!self.failed);

        let headers = &self.response
                           .as_ref()
                           .expect("Resource hasn't been downloaded yet")
                           .headers;

        headers.get().cloned()
    }

    pub fn get_response_url(&self) -> &Url {
        assert!(!self.failed);

        &self.response
             .as_ref()
             .expect("Resource hasn't been downloaded yet")
             .url
    }

    pub fn read_response(&mut self) -> &[u8] {
        assert!(!self.failed);

        // let mut buffer = Vec::new();
        &self.response
             .as_mut()
             .unwrap()
             .body
    }
}
