// FIXME: VTune reports many spinlocks. Look into it...
// FIXME: Profile with coz
// TODO: Figure out user interface


#![feature(plugin)]
#![plugin(clippy)]
#![plugin(phf_macros)]

extern crate chan;
extern crate crypto;
extern crate cssparser;
extern crate html5ever;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate log4rs;
#[macro_use]
extern crate mime;
extern crate num_cpus;
extern crate phf;
extern crate rand;
extern crate rusqlite;
extern crate tendril;
extern crate time;
extern crate url;


use std::thread;

use event::Event;
use task::Task;


mod download;
mod event;
mod model;
mod parser;
mod storage;
mod task;


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


fn main() {
    log4rs::init_file("logging.toml", Default::default()).unwrap();
    debug!("Starting up");
    let start_time = time::now();

    // Set up thread communication
    let (task_tx, task_rx) = chan::async();
    let (evt_tx, evt_rx) = chan::async();

    // Set up website map
    // TODO: Replace with db access
    let websites = storage::load_queued();


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

    event::event_handler(evt_rx, Queues::new(task_tx, evt_tx), websites, thread_count);

    // FIXME: Add thread with current state (# of downloads, resources, ...)

    // Wait for threads to finish
    for thread in threads.into_iter() {
        thread.join().unwrap();
    }

    print!("Done. Run time: ");
    print_duration(time::now() - start_time);
}


fn print_duration(duration: time::Duration) {
    let days = duration.num_days();
    let mut hours = duration.num_hours();
    let mut minutes = duration.num_minutes();
    let mut seconds = duration.num_seconds();

    if days > 0 {
        print!("{}d ", days);
        hours -= days * 24;
        minutes -= hours * 60 * 24;
        seconds -= seconds * 60 * 60 * 24;
    }

    if hours > 0 {
        print!("{}h ", hours);
        minutes -= hours * 60;
        seconds -= seconds * 60 * 60;
    }

    if minutes > 0 {
        print!("{}h ", minutes);
        seconds -= seconds * 60;
    }

    println!("{}s", seconds);
}
