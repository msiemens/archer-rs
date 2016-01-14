// FIXME: VTune reports many spinlocks. Look into it...
// FIXME: Profile with coz
// TODO: Figure out user interface


#![feature(plugin)]
#![plugin(clippy)]
#![plugin(phf_macros)]

extern crate chan;
extern crate cssparser;
extern crate html5ever;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate log4rs;
#[macro_use]
extern crate mime;
extern crate num_cpus;
extern crate phf;
extern crate rand;
// extern crate rusqlite;
extern crate tendril;
extern crate time;
extern crate url;


use std::collections::HashMap;
use std::thread;
use std::sync::RwLock;

use event::Event;
use model::{Website, State, WebsiteID};
use task::Task;


mod download;
mod event;
mod model;
mod parser;
mod task;


const SITES: [&'static str; 3] = ["http://m-siemens.de",
                                  "http://blog.m-siemens.de",
                                  "http://rust-lang.org/"];

pub type EventQueue = chan::Sender<Event>;
pub type TaskQueue = chan::Sender<Task>;

#[derive(Clone)]
pub struct Queues {
    tasks: TaskQueue,
    events: EventQueue,
}

impl Queues {
    pub fn new(tasks: TaskQueue, events: EventQueue) -> Queues {
        Queues {
            tasks: tasks,
            events: events,
        }
    }

    pub fn send_task(&self, task: Task) {
        self.tasks.send(task);
    }

    pub fn send_event(&self, event: Event) {
        self.events.send(event);
    }
}


// FIXME: Split this up!
fn main() {
    log4rs::init_file("logging.toml", Default::default()).unwrap();
    debug!("Starting up");

    // Set up thread communication
    let (task_tx, task_rx) = chan::async();
    let (evt_tx, evt_rx) = chan::async();

    // Set up website map
    // TODO: Replace with db access
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


    // Fill queue with initial tasks
    debug!("Filling up task queue initially");

    for (i, website) in &websites {
        task_tx.send(Task::Download {
            wid: *i,
            url: website.url.clone(),
            retry: 0,
            resource: false,
        })
    }

    let websites = RwLock::new(websites);

    // Set up worker pool
    let mut threads: Vec<_> = vec![];
    let evt_rx = evt_rx.clone();



    // FIXME: Make configurable
    // let thread_count = 1;
    let thread_count = num_cpus::get();

    debug!("Starting execution with {} threads", thread_count);

    // Start threads
    for tid in 0..thread_count {
        let task_rx = task_rx.clone();
        let queues = Queues::new(task_tx.clone(), evt_tx.clone());
        let thread = thread::Builder::new()
                         .name(tid.to_string())
                         .spawn(move || {
                             for task in task_rx.iter() {
                                 if let Task::Terminate = task {
                                     debug!("Thread terminating");
                                     break;
                                 } else {
                                     task::dispatch_task(task, queues.clone())
                                 }
                             }
                         });

        threads.push(thread.unwrap());
    }

    // Small helper for event handler
    let quit_if_done = || {
        let websites = websites.read().unwrap();

        trace!("quit_if_done - websites: {:?}", *websites);

        if websites.values().all(|w| w.state != State::InProgress) {
            debug!("all websites done/failed");

            for _ in 0..thread_count {
                // Terminate all tasks
                task_tx.send(Task::Terminate);
                evt_tx.send(Event::Terminate);
            }
        }
    };

    // Event handler (main thread)
    for event in evt_rx.iter() {
        debug!("Handling event: {:?}", event);

        match event {
            Event::DownloadProcessed { wid, resource, explored } => {
                {
                    let mut websites = websites.write().unwrap();
                    let mut website = websites.get_mut(&wid)
                                              .expect("Invalid website ID");

                    // If the website hasn't been explored yet and there
                    // are no resources, we're done.
                    if !resource && website.explored == false && explored == 0 {
                        website.state = State::Done;

                        info!("Site {} downloaded", website.url);
                    } else {
                        // Otherwise update explored resources count
                        website.explored = true;
                        website.resources_explored += explored;
                        if resource {
                            website.resources_downloaded += 1;
                        }

                        // Update the website state if all resources are downloaded
                        if website.resources_explored == website.resources_downloaded {
                            website.state = State::Done;

                            info!("Site {} downloaded", website.url);
                        }
                    }
                }

                // Check if we're done
                quit_if_done();
            }
            Event::DownloadFailed { wid, url } => {
                {
                    let mut websites = websites.write().unwrap();
                    let mut website = websites.get_mut(&wid)
                                              .expect("Invalid website ID");

                    // Update the website state
                    website.state = State::Failed;

                    warn!("Downloading {} failed, couldn't download {}",
                          website.url,
                          url);
                }

                // Check if we're done
                quit_if_done();
            }
            Event::Terminate => break,
        }
    }

    // FIXME: Add thread with current state (# of downloads, resources, ...)

    // Wait for threads to finish
    for thread in threads.into_iter() {
        thread.join().unwrap();
    }

    println!("Done");
}
