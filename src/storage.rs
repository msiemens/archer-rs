use std::collections::HashMap;
use std::path::Path;

use mime::Mime;
use rusqlite::Connection;
use time::Timespec;
use url::Url;

use Queues;
use event::Event;
use model::{State, Website, WebsiteID};


const SITES: [&'static str; 3] = ["http://m-siemens.de",
                                  "http://blog.m-siemens.de",
                                  "http://rust-lang.org/"];


pub fn load_queued() -> HashMap<WebsiteID, Website> {
    let mut websites = HashMap::new();
    for (i, website) in SITES.iter().enumerate() {
        websites.insert(WebsiteID::new(i as i32),
                        Website {
                            url: (*website).into(),
                            resources_explored: 0,
                            resources_downloaded: 0,
                            resources_stored: 0,
                            state: State::InProgress,
                            explored: false,
                        });
    }

    websites
}


pub fn store_resource(wid: WebsiteID,
                      url: Url,
                      data: Vec<u8>,
                      mime_type: Option<Mime>,
                      timestamp: Timespec,
                      queues: Queues) {
    info!("Storing resource: {}", url);

    with_connection(|conn| {
        // FIXME: Error handling, handle SQL_BUSY
        let tx = try!(conn.transaction());

        try!(conn.execute("INSERT INTO resource (url, contents, mime, time) VALUES ($1, $2, $3, \
                           $4)",
                          &[&url.serialize(),
                            &data,
                            &mime_type.as_ref().map(|mime| format!("{}", mime)),
                            &timestamp]));

        tx.commit()
    })
        .unwrap();

    queues.send_event(Event::DownloadStored { wid: wid });
}


fn with_connection<T, F: Fn(&Connection) -> T>(f: F) -> T {
    // FIXME: Make path configurable
    thread_local! {
        static CONNECTION: Connection = Connection::open(Path::new("archer.db")).unwrap()
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

        f(conn)
    })
}
