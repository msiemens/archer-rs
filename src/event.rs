use std::collections::HashMap;

use chan;

use Queues;
use model::{State, Website, WebsiteID};
use task::Task;


#[derive(Debug)]
pub enum Event {
    DownloadProcessed {
        wid: WebsiteID,
        resource: bool,
        explored: i32,
    },
    DownloadStored {
        wid: WebsiteID,
    },
    DownloadFailed {
        wid: WebsiteID,
        url: String,
    },
    Terminate,
}


/// Event handler (running in the main thread)
pub fn event_handler(evt_rx: chan::Receiver<Event>,
                     queues: Queues,
                     mut websites: HashMap<WebsiteID, Website>,
                     thread_count: usize) {

    for event in evt_rx.iter() {
        debug!("Handling event: {:?}", event);

        match event {
            // FIXME: Only exit when data has been stored!?
            Event::DownloadProcessed { wid, resource, explored } => {
                let mut website = websites.get_mut(&wid)
                                          .expect("Invalid website ID");

                if resource {
                    debug_assert!(website.explored);

                    website.resources_downloaded += 1;
                }

                match (resource, explored) {
                    (false, 0) => {
                        // No resources explored on this website. We're done
                        assert!(!resource);

                        website.state = State::Done;
                        info!("Site {} downloaded", website.url);
                    }
                    (_, _) => {
                        // There are explored resources on this website/resource
                        website.explored = true;
                        website.resources_explored += explored;

                        if website.resources_explored == website.resources_downloaded {
                            info!("Site {} downloaded", website.url);
                        }
                    }
                }
            }
            Event::DownloadStored { wid } => {
                let mut website = websites.get_mut(&wid)
                                          .expect("Invalid website ID");

                // Update the website state if all resources are downloaded and stored
                if website.resources_explored == website.resources_downloaded &&
                   website.resources_explored == website.resources_stored {
                    website.state = State::Done;
                }
            }
            Event::DownloadFailed { wid, url } => {
                let mut website = websites.get_mut(&wid)
                                          .expect("Invalid website ID");

                // Update the website state
                website.state = State::Failed;

                warn!("Downloading {} failed, couldn't download {}",
                      website.url,
                      url);
            }
            Event::Terminate => break,
        }

        // Check wheter we're done
        if websites.values().all(|w| w.state != State::InProgress) {
            debug!("all websites done/failed");

            for _ in 0..thread_count {
                // Terminate all tasks
                queues.send_task(Task::Terminate);
                queues.send_event(Event::Terminate);
                // FIXME: Is Event::Terminate needed?
            }
        }
    }
}
