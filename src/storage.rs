use std::path::Path;

use rusqlite::Connection;
use url::Url;

use tasks::Resource;


// DEV only: hardcoded sites to download
const SITES: [&'static str; 3] = ["http://m-siemens.de",
                                  "http://blog.m-siemens.de/",
                                  "http://rust-lang.org/"];


pub fn resource_exists(url: &Url) -> bool {
    with_connection(|conn| {
        conn.query_row("SELECT EXISTS(SELECT * FROM resource WHERE url=$1)",
                       &[&(url.serialize())],
                       |row| row.get(0))
    })
        .unwrap()  // FIXME: Error handling
}


pub fn load_queued() -> Vec<String> {
    let mut websites = Vec::new();
    for website in &SITES {
        websites.push((*website).into());
    }

    websites
}


pub fn store_resource(resource: &mut Resource) {
    {
        let details = resource.get_storage_details();

        debug!("Storing resource: {}", resource.get_url());

        with_connection(|conn| {
            let tx = try!(conn.transaction());

            try!(conn.execute("INSERT INTO resource (url,contents,mime,time) VALUES \
                               ($1,$2,$3,$4)",
                              &[&resource.get_url().serialize(),
                                &details.contents,
                                &details.mime.as_ref().map(|mime| format!("{}", mime)),
                                details.timestamp]));

            tx.commit()
        })
            .unwrap();  // FIXME: Error handling
    }

    resource.stored();
}


pub fn purge_resources() {
    with_connection(|conn| {
        let tx = try!(conn.transaction());

        try!(conn.execute("DROP TABLE IF EXISTS resource", &[]));

        tx.commit()
    })
        .unwrap();
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

        f(conn)
    })
}
