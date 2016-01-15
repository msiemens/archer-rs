use std::time::Duration;
use std::thread::sleep;

use hyper::Error;
use hyper::client::Client;
use rand;
use rand::distributions::{IndependentSample, Range};

use Queues;
use event::Event;
use model::WebsiteID;
use task::Task;


const MAX_RETRIES: i32 = 5;

// Every thread gets its own client. This minimizes synchronization overhead at
// the cost of reusing connections.
thread_local!(static HTTP: Client = Client::new());


pub fn task_download(wid: WebsiteID, url: String, retries: i32, resource: bool, queues: Queues) {
    // FIXME: Handling of redirects
    // FIXME: Handling of gzipped content
    // TODO: Add user-agent header
    // TODO: robots.txt handling
    if resource {
        info!("Downloading resource {}", url);
    } else {
        info!("Downloading website {}", url);
    }

    HTTP.with(|http| {
        match http.get(&url).send() {
            Ok(resp) => {
                queues.send_task(Task::ParseResource {
                    wid: wid,
                    data: resp,
                    resource: resource,
                });
            }
            Err(Error::Io(err)) => {
                if retries >= MAX_RETRIES {
                    println!("ERROR: Giving up on {} after {} retries: {}",
                             url,
                             retries,
                             err);
                    queues.send_event(Event::DownloadFailed {
                        wid: wid,
                        url: url,
                    })
                } else {
                    // Back off and retry
                    let rng_range = Range::new(0, 1000);
                    let mut rng = rand::thread_rng();

                    let backoff_time = 1000 + rng_range.ind_sample(&mut rng);
                    sleep(Duration::from_millis(backoff_time));

                    // Re-submit URL
                    queues.send_task(Task::Download {
                        wid: wid,
                        url: url,
                        retry: retries + 1,
                        resource: resource,
                    });
                }
            }
            Err(err) => {
                // TODO: Better error handling
                println!("ERROR: Could not download {}: {:?}", url, err);

                queues.send_event(Event::DownloadFailed {
                    wid: wid,
                    url: url,
                })
            }
        }
    });
}
