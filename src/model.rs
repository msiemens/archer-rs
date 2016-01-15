#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    InProgress,
    Done,
    Failed,
}


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WebsiteID(i32);

impl WebsiteID {
    pub fn new(id: i32) -> WebsiteID {
        WebsiteID(id)
    }
}


#[derive(Debug)]
pub struct Website {
    // pub id: i32,
    pub url: String,
    pub explored: bool,
    pub resources_explored: i32,
    pub resources_downloaded: i32,
    pub state: State,
}
