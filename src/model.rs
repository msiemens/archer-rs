// use mime::Mime;
// use rusqlite::{Connection, Result};
// use time::Timespec;


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

// impl Website {
// fn all(conn: &Connection) -> Result<Vec<Website>> {
// let mut stmt = conn.prepare("SELECT id, url, downloaded FROM website")
// .unwrap();
//
// let rows = try!(stmt.query_map(&[], |row| {
// Website {
// id: row.get(0),
// url: row.get(1),
// downloaded: row.get(2),
// }
// }));
//
// rows.collect()
// }
//
// fn insert(&self, conn: &Connection) -> Result<i32> {
// conn.execute_named("INSERT INTO website (url, downloaded) VALUES (:url, :downloaded)",
// &[(":url", &self.url), (":downloaded", &self.downloaded)])
// }
//
// fn update(&self, conn: &Connection) -> Result<i32> {
// conn.execute_named("UPDATE website SET (url, downloaded) VALUES (:url, :downloaded) WHERE \
// id = (:id)",
// &[(":id", &self.id),
// (":url", &self.url),
// (":downloaded", &self.downloaded)])
// }
// }
//
//
//
// #[derive(Debug)]
// struct Tag {
// id: i32,
// name: String,
// slug: String,
// }
//
//
// #[derive(Debug)]
// struct Resource {
// id: i32,
// original_url: String,
// rewritten_url: String,
// contents: Vec<u8>,
// mime: Mime,
// time: Timespec,
// }
//
