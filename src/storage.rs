use std::collections::HashMap;

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
                            state: State::InProgress,
                            explored: false,
                        });
    }

    websites
}
