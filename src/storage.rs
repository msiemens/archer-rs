use std::collections::HashMap;
use std::path::Path;

use rusqlite::Connection;
use url::Url;

use tasks::Resource;
use server::overview::{OverviewData, WebsiteData, TagData};


// DEV only: hardcoded sites to download
// const SITES: [&'static str; 3] = ["http://m-siemens.de",
//                                   "http://blog.m-siemens.de/",
//                                   "http://rust-lang.org/"];


pub fn resource_exists(url: &Url) -> bool {
    with_connection(|conn| {
        conn.query_row("SELECT EXISTS(SELECT * FROM resource WHERE url=$1)",
                       &[&(url.serialize())],
                       |row| row.get(0))
            .unwrap()
    })
}


pub fn load_queued() -> Vec<String> {
    with_connection(|conn| {
        let mut stmt = conn.prepare("SELECT url FROM websites WHERE downloaded = 0").unwrap();

        stmt.query_map(&[], |row| row.get::<String>(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    })
}


pub fn store_resource(resource: &mut Resource) {
    {
        let details = resource.get_storage_details();

        debug!("Storing resource: {}", resource.get_url());

        with_connection(|conn| {
            conn.execute("INSERT INTO resource (url,contents,mime,time) VALUES \
                               ($1,$2,$3,$4)",
                         &[&resource.get_url().serialize(),
                           &details.contents,
                           &details.mime.as_ref().map(|mime| format!("{}", mime)),
                           details.timestamp])
                .unwrap();
        })
    }

    resource.stored();
}


pub fn purge_resources() {
    with_connection(|conn| {
        conn.execute("DROP TABLE IF EXISTS resource", &[]).unwrap();
    })
}


pub fn get_overview() -> OverviewData {
    with_connection(|conn| {
        let mut tags = HashMap::new();
        let mut websites = HashMap::new();

        // Get tags
        let mut stmt = conn.prepare("SELECT id, name, color FROM tags").unwrap();
        stmt.query_map(&[], |row| {
                tags.insert(row.get::<String>(0),
                            TagData {
                                name: row.get(1),
                                color: row.get(2),
                            })
            })
            .unwrap();

        // Get websites
        let mut stmt = conn.prepare("SELECT id, url FROM websites").unwrap();
        stmt.query_map(&[], |row| {
                websites.insert(row.get::<i32>(0),
                                WebsiteData {
                                    title: "".into(),
                                    url: row.get(1),
                                    tags: Vec::new(),
                                })
            })
            .unwrap();

        // For every website: get associated tags
        for (id, website) in &mut websites {
            let mut stmt = conn.prepare("SELECT tagid FROM websites_tags WHERE websiteid = ?")
                               .unwrap();
            website.tags.extend(stmt.query_map(&[id], |row| row.get::<i32>(0))
                                    .unwrap()
                                    .map(Result::unwrap))
        }

        OverviewData {
            tags: tags,
            websites: websites.into_iter().map(|(_, website)| website).collect(),
        }
    })
}


fn with_connection<T, F: Fn(&Connection) -> T>(f: F) -> T {
    // TODO: Make path configurable
    thread_local! {
        static CONNECTION: Connection = Connection::open(Path::new("archer.db")).unwrap()
    //  static CONNECTION: Connection = Connection::open_in_memory().unwrap()
    };

    CONNECTION.with(|conn| {
        conn.execute(r"
            CREATE TABLE IF NOT EXISTS resource (
              id       INTEGER PRIMARY KEY,
              url      TEXT NOT NULL,
              contents BLOB NOT NULL,
              mime     TEXT,
              time     INTEGER NOT NULL
              )",
                     &[])
            .unwrap();

        conn.execute(r"
            CREATE TABLE IF NOT EXISTS websites (
              id         INTEGER PRIMARY KEY,
              url        TEXT NOT NULL,
              downloaded BOOLEAN NOT NULL
              )",
                     &[])
            .unwrap();

        conn.execute(r"
            CREATE TABLE IF NOT EXISTS tags (
              id         INTEGER PRIMARY KEY,
              name       TEXT NOT NULL,
              color      TEXT NOT NULL
              )",
                     &[])
            .unwrap();

        conn.execute(r"
            CREATE TABLE IF NOT EXISTS websites_tags (
              websiteid  INTEGER,
              tagid      INTEGER,
              FOREIGN KEY(websiteid) REFERENCES website(id),
              FOREIGN KEY(tagid) REFERENCES tags(id) 
              )",
                     &[])
            .unwrap();

        f(conn)
    })
}
