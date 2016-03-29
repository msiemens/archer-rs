use std::io::Read;
use std::io::Error as IOError;
use std::time::Duration;
use std::thread::sleep;

use hyper::Error;
use hyper::client::Client;
use hyper::header::Headers;
use hyper::status::StatusCode;
use rand;
use rand::distributions::{IndependentSample, Range};
use time;
use url::Url;

use storage;
pub use self::pool::DownloadPool as Pool;


mod pool;


#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: Headers,
    pub url: Url,
    pub body: Vec<u8>,
}


const MAX_RETRIES: i32 = 5;

// Every thread gets its own client. This minimizes synchronization overhead at
// the cost of not reusing connections between threads.
thread_local!(static HTTP: Client = Client::new());


fn get_headers() -> Headers {
    // let mut headers = Headers::new();
    // headers.set(AcceptEncoding(vec![qitem(Encoding::Deflate), qitem(Encoding::Gzip)]));
    // headers

    Headers::new()
}


#[derive(Debug)]
pub enum DownloadResult {
    Success {
        response: Response,
        time: time::Timespec,
    },
    Skip,
    Retry,
    Failed,
}


pub fn download_url(try: i32, url: &Url) -> DownloadResult {
    // FIXME: Handling of redirects
    // FIXME: Handling of gzipped content
    // TODO: Add user-agent header
    // TODO: robots.txt handling?

    if storage::resource_exists(url) {
        debug!("Resource {:?} already downloaded, skipping", url);
        return DownloadResult::Skip;
    }

    // TODO: Print 'website' if it's a website
    info!("Downloading resource {}", url);


    HTTP.with(|http| {
        match http.get(&url.serialize()).headers(get_headers()).send() {
            Ok(mut resp) => {
                if resp.status != StatusCode::Ok {
                    error!("Could not download {}: {}", url, resp.status);
                    return DownloadResult::Failed;
                }

                // Try reading the full response
                let mut body = Vec::new();
                match resp.read_to_end(&mut body) {
                    Ok(_) => {
                        DownloadResult::Success {
                            response: Response {
                                status: resp.status,
                                headers: resp.headers.clone(),
                                url: resp.url.clone(),
                                body: body,
                            },
                            time: time::get_time(),
                        }
                    }
                    Err(err) => retry(try, &url, err),
                }
            }

            // Handle I/O errors
            Err(Error::Io(err)) => retry(try, &url, err),

            // Handle other errors (mainly invalid HTTP response)
            Err(err) => {
                error!("Could not download {}: {:?}", url, err);
                DownloadResult::Failed
            }
        }
    })
}


fn retry(try: i32, url: &Url, err: IOError) -> DownloadResult {
    if try >= MAX_RETRIES {
        error!("Giving up on {} after {} retries: {}", url, try, err);
        DownloadResult::Failed
    } else {
        // Back off and retry at least 1 sec later
        // TODO: We block a whole thread here. Can we do better?
        info!("I/O error while downloading {}... Retrying soon", url);

        let rng_range = Range::new(0, 1000);
        let mut rng = rand::thread_rng();

        let backoff_time = 1000 + rng_range.ind_sample(&mut rng);
        sleep(Duration::from_millis(backoff_time));

        // Retry
        DownloadResult::Retry
    }
}
